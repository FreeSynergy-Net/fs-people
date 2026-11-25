//! The commands against a real server on a real socket (in this process).

use std::sync::Arc;

use fs_interfaces::people_capnp::people;
use fs_rpc::{bind_socket, connect, serve};
use fs_objects::{Store, TursoStore};
use fs_people_cli::{Command, run};
use fs_people_core::PeopleRepo;
use fs_people_server::PeopleServer;

async fn with_cli<F, Fut>(test: F)
where
    F: FnOnce(people::Client) -> Fut,
    Fut: Future<Output = ()>,
{
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let store: Arc<dyn Store> = Arc::new(TursoStore::open_in_memory().await.unwrap());
            let repo = Arc::new(PeopleRepo::new(store, "https://local.invalid/users/owner"));
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("people.sock");
            let root = PeopleServer::new(repo.clone(), repo.clone(), repo).into_client();
            let bound = bind_socket(&path).await.unwrap();
            let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
            let serving = tokio::task::spawn_local(serve(bound, root.client, async {
                let _ = stopped.await;
            }));
            let connection = connect::<people::Client>(&path).await.unwrap();
            test(connection.client.clone()).await;
            drop(connection);
            let _ = stop.send(());
            let _ = serving.await;
        })
        .await;
}

fn add(name: &str, email: &str) -> Command {
    Command::Add {
        name: name.into(),
        email: Some(email.into()),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn add_list_find_show_export_delete() {
    with_cli(|root| async move {
        let amara = run(&root, add("Amara", "amara@example.org")).await.unwrap();
        run(&root, add("Mirela", "mirela@example.org"))
            .await
            .unwrap();
        let id = amara.split("  ").next().unwrap().to_string();
        assert!(amara.ends_with("  Amara  amara@example.org"), "{amara}");

        let all = run(&root, Command::List(None)).await.unwrap();
        assert_eq!(all.lines().count(), 2, "{all}");
        let some = run(&root, Command::List(Some("mirela".into())))
            .await
            .unwrap();
        assert!(some.contains("Mirela") && !some.contains("Amara"), "{some}");

        let found = run(&root, Command::Find("amara@example.org".into()))
            .await
            .unwrap();
        assert_eq!(found, amara);

        let shown = run(&root, Command::Show(id.clone())).await.unwrap();
        assert!(shown.contains("\"full\": \"Amara\""), "{shown}");
        let vcard = run(&root, Command::Export(id.clone())).await.unwrap();
        assert!(
            vcard.contains("BEGIN:VCARD") && vcard.contains("Amara"),
            "{vcard}"
        );

        assert_eq!(
            run(&root, Command::Delete(id.clone())).await.unwrap(),
            format!("deleted {id}")
        );
        assert!(run(&root, Command::Show(id)).await.is_err());
        assert!(run(&root, Command::Show("not-an-id".into())).await.is_err());
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn import_reports_created_then_skipped() {
    with_cli(|root| async move {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("in.vcf");
        std::fs::write(
            &file,
            "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:01a1b91a-0000-7000-8000-000000000001\r\nFN:Teodor\r\nEND:VCARD\r\n",
        )
        .unwrap();
        let first = run(&root, Command::Import(file.clone())).await.unwrap();
        assert_eq!(first, "created 1, skipped 0, failed 0");
        let again = run(&root, Command::Import(file)).await.unwrap();
        assert_eq!(again, "created 0, skipped 1, failed 0");
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn an_import_file_that_cannot_be_read_is_an_error_naming_it() {
    with_cli(|root| async move {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.txt");
        let got = run(&root, Command::Import(missing.clone())).await;
        assert!(got.unwrap_err().contains("nope.txt"));
        let binary = dir.path().join("bin.dat");
        std::fs::write(&binary, [0xff, 0xfe, 0x00]).unwrap();
        let got = run(&root, Command::Import(binary)).await;
        assert!(got.unwrap_err().contains("not UTF-8"));
    })
    .await;
}
