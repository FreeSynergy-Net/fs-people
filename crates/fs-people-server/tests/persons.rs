//! Cards and lookups of the contract over a real socket, against the store-backed provider.

mod common;

use common::{CLUB, card, create, id_of, json_of, name_of, persons_of, roles_of, with_server};
use fs_interfaces::people_capnp::persons;
use fs_interfaces::rpc::write_uuid;
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn create_get_update_delete() {
    with_server(|root| async move {
        let persons = persons_of(&root).await;
        let id = create(&persons, "uid-1", "Amara", "amara@x.org").await;

        let mut get = persons.get_request();
        write_uuid(get.get().init_id(), Some(id));
        let got = get.send().promise.await.unwrap();
        let got = got.get().unwrap();
        assert!(got.get_found());
        assert_eq!(name_of(got.get_person().unwrap()), "Amara");
        assert_eq!(json_of(got.get_person().unwrap())["uid"], "uid-1");

        let mut by_uid = persons.get_by_uid_request();
        by_uid.get().set_uid("uid-1");
        let got = by_uid.send().promise.await.unwrap();
        let got = got.get().unwrap();
        assert!(got.get_found());
        assert_eq!(id_of(got.get_person().unwrap()), id);
        assert_eq!(name_of(got.get_person().unwrap()), "Amara");
        assert_eq!(
            json_of(got.get_person().unwrap())["emails"]["e1"]["address"],
            "amara@x.org"
        );

        let mut update = persons.update_request();
        write_uuid(update.get().init_id(), Some(id));
        update
            .get()
            .set_jscontact(card("uid-1", "Amara Brandt", "amara@x.org").as_str());
        let updated = update.send().promise.await.unwrap();
        assert_eq!(
            name_of(updated.get().unwrap().get_person().unwrap()),
            "Amara Brandt"
        );

        let mut delete = persons.delete_request();
        write_uuid(delete.get().init_id(), Some(id));
        delete.send().promise.await.unwrap();

        let mut get = persons.get_request();
        write_uuid(get.get().init_id(), Some(id));
        let gone = get.send().promise.await.unwrap();
        assert!(!gone.get().unwrap().get_found());
        let mut by_uid = persons.get_by_uid_request();
        by_uid.get().set_uid("uid-1");
        let gone = by_uid.send().promise.await.unwrap();
        assert!(!gone.get().unwrap().get_found());
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn find_by_describes_email_phone() {
    with_server(|root| async move {
        let persons = persons_of(&root).await;
        let text = json!({"@type": "Card", "version": "1.0", "uid": "uid-1",
            "name": {"full": "Amara"},
            "emails": {"e": {"address": "Amara@X.org"}},
            "phones": {"p": {"number": "+49 30 1234"}},
            "freesynergy.net:describes": "did:example:amara"});
        let mut req = persons.create_request();
        req.get().set_jscontact(text.to_string().as_str());
        let answer = req.send().promise.await.unwrap();
        let id = id_of(answer.get().unwrap().get_person().unwrap());
        create(&persons, "uid-2", "Bob", "bob@x.org").await;

        let mut req = persons.find_by_describes_request();
        req.get().set_iri("did:example:amara");
        let found = req.send().promise.await.unwrap();
        let items = found.get().unwrap().get_items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(id_of(items.get(0)), id);

        let mut req = persons.find_by_email_request();
        req.get().set_email("amara@x.ORG");
        let found = req.send().promise.await.unwrap();
        let items = found.get().unwrap().get_items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(id_of(items.get(0)), id);

        let mut req = persons.find_by_phone_request();
        req.get().set_phone("+49 (30) 1234");
        let found = req.send().promise.await.unwrap();
        let items = found.get().unwrap().get_items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(id_of(items.get(0)), id);

        // Other values find nobody; the other person is not among any of these.
        let mut req = persons.find_by_email_request();
        req.get().set_email("nobody@x.org");
        let found = req.send().promise.await.unwrap();
        assert_eq!(found.get().unwrap().get_items().unwrap().len(), 0);
        let mut req = persons.find_by_phone_request();
        req.get().set_phone("+49 30 9999");
        let found = req.send().promise.await.unwrap();
        assert_eq!(found.get().unwrap().get_items().unwrap().len(), 0);
        let mut req = persons.find_by_describes_request();
        req.get().set_iri("did:example:bob");
        let found = req.send().promise.await.unwrap();
        assert_eq!(found.get().unwrap().get_items().unwrap().len(), 0);
    })
    .await;
}

async fn listed(persons: &persons::Client, kind: &str, role: &str, term: &str) -> Vec<String> {
    let mut req = persons.list_request();
    let mut filter = req.get().init_filter();
    filter.set_kind(kind);
    filter.set_role(role);
    filter.set_term(term);
    let answer = req.send().promise.await.unwrap();
    let page = answer.get().unwrap().get_page().unwrap();
    page.get_items().unwrap().iter().map(name_of).collect()
}

#[tokio::test(flavor = "current_thread")]
async fn the_filter_combines_with_and() {
    with_server(|root| async move {
        let persons = persons_of(&root).await;
        let roles = roles_of(&root).await;
        let amara = create(&persons, "uid-1", "Amara", "amara@x.org").await;
        create(&persons, "uid-2", "Amarant", "amarant@x.org").await;
        create(&persons, "uid-3", "Bob", "bob@x.org").await;
        let mut list = persons.create_list_request();
        list.get().set_name("Amara fans");
        list.send().promise.await.unwrap();

        let mut assign = roles.assign_request();
        write_uuid(assign.get().init_person(), Some(amara));
        assign.get().set_role("member");
        assign.get().set_scope(CLUB);
        assign.send().promise.await.unwrap();

        assert_eq!(listed(&persons, "", "", "").await.len(), 4);
        assert_eq!(
            listed(&persons, "", "", "amara").await,
            ["Amara", "Amara fans", "Amarant"]
        );
        assert_eq!(
            listed(&persons, "individual", "", "amara").await,
            ["Amara", "Amarant"]
        );
        assert_eq!(listed(&persons, "group", "", "").await, ["Amara fans"]);
        assert_eq!(listed(&persons, "", "member", "").await, ["Amara"]);
        assert_eq!(
            listed(&persons, "individual", "member", "bob").await,
            [""; 0]
        );
    })
    .await;
}
