//! B8: a proposal the user turned down does not come back, also after the server was restarted, and
//! taking a link back (unlink) counts as a no. Over a real socket and a store FILE that is opened a
//! second time (the restart).
//!
//! Pattern: Round trip with its own oracle (the lists of proposals before and after).
//! Mutant: the filter `rejected.contains` in `repo/proposals.rs` removed: the pair is proposed again
//! after the restart and the second test is red.

mod common;

use std::sync::Arc;

use common::{ME, create, persons_of, with_root};
use fs_objects::{Store, TursoStore};
use fs_people_core::PeopleRepo;
use fs_people_server::PeopleServer;
use fs_rpc::{read_uuid, write_uuid};

async fn serve_file<F, Fut>(path: &std::path::Path, test: F)
where
    F: FnOnce(fs_interfaces::people_capnp::people::Client) -> Fut,
    Fut: Future<Output = ()>,
{
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let store: Arc<dyn Store> = Arc::new(TursoStore::open(path).await.unwrap());
            let repo = Arc::new(PeopleRepo::new(store, ME));
            let root = PeopleServer::new(repo.clone(), repo.clone(), repo).into_client();
            with_root(root, test).await;
        })
        .await;
}

async fn proposals(persons: &fs_interfaces::people_capnp::persons::Client) -> usize {
    let answer = persons.proposals_request().send().promise.await.unwrap();
    answer.get().unwrap().get_items().unwrap().len() as usize
}

#[tokio::test(flavor = "current_thread")]
async fn a_rejected_pair_is_not_proposed_again_after_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("people.db");
    serve_file(&file, |root| async move {
        let persons = persons_of(&root).await;
        let a = create(&persons, "uid-1", "Amara", "amara@x.org").await;
        let b = create(&persons, "uid-2", "A. King", "amara@x.org").await;
        assert_eq!(proposals(&persons).await, 1);
        let mut reject = persons.reject_same_request();
        write_uuid(reject.get().init_a(), Some(a));
        write_uuid(reject.get().init_b(), Some(b));
        reject.send().promise.await.unwrap();
        assert_eq!(proposals(&persons).await, 0);
    })
    .await;
    serve_file(&file, |root| async move {
        let persons = persons_of(&root).await;
        assert_eq!(
            proposals(&persons).await,
            0,
            "still not asked after the restart"
        );
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn taking_a_link_back_counts_as_a_no_also_after_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("people.db");
    serve_file(&file, |root| async move {
        let persons = persons_of(&root).await;
        let a = create(&persons, "uid-1", "Amara", "amara@x.org").await;
        let b = create(&persons, "uid-2", "A. King", "amara@x.org").await;
        let mut confirm = persons.confirm_same_request();
        write_uuid(confirm.get().init_a(), Some(a));
        write_uuid(confirm.get().init_b(), Some(b));
        let answer = confirm.send().promise.await.unwrap();
        let link = {
            let relationship = answer.get().unwrap().get_link().unwrap();
            read_uuid(relationship).unwrap()
        };
        let mut unlink = persons.unlink_same_request();
        write_uuid(unlink.get().init_link(), Some(link));
        unlink.send().promise.await.unwrap();
        assert_eq!(proposals(&persons).await, 0, "a taken back link is a no");
    })
    .await;
    serve_file(&file, |root| async move {
        let persons = persons_of(&root).await;
        assert_eq!(proposals(&persons).await, 0);
    })
    .await;
}
