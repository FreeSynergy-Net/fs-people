//! The role calls against the server.
//!
//! Pattern: Adapter — each role question as a request on the server's roles capability.

use fs_interfaces::people_capnp::{RoleScope as Wire, roles};
use fs_objects::ObjectId;
use fs_people_core::{Result, RoleAssignment, RoleDefinition, RoleScope};
use fs_rpc::{read_object_id, write_uuid};

use super::convert::problem;

pub(super) async fn roles_of(roles: &roles::Client, id: ObjectId) -> Result<Vec<RoleAssignment>> {
    let mut request = roles.roles_of_request();
    write_uuid(request.get().init_person(), Some(id.as_uuid()));
    let answer = request
        .send()
        .promise
        .await
        .map_err(|e| problem(&e, Some(id)))?;
    let items = answer
        .get()
        .and_then(roles::roles_of_results::Reader::get_items)
        .map_err(|e| problem(&e, Some(id)))?;
    let mut held = Vec::new();
    for a in items {
        let (Ok(assignment), Ok(person)) = (
            a.get_id().and_then(read_object_id),
            a.get_person().and_then(read_object_id),
        ) else {
            continue;
        };
        let scope = a
            .get_scope()
            .and_then(|t| t.to_string().map_err(Into::into))
            .map_err(|e: capnp::Error| problem(&e, Some(id)))?;
        held.push(RoleAssignment {
            id: assignment,
            person,
            role: a
                .get_role()
                .and_then(|t| t.to_string().map_err(Into::into))
                .map_err(|e: capnp::Error| problem(&e, Some(id)))?,
            scope: (!scope.is_empty()).then_some(scope),
            details: serde_json::Map::new(),
        });
    }
    Ok(held)
}

/// What the window needs of a role: its id and its names.
pub(super) async fn role(roles: &roles::Client, id: &str) -> Result<Option<RoleDefinition>> {
    let mut request = roles.get_request();
    request.get().set_id(id);
    let answer = request
        .send()
        .promise
        .await
        .map_err(|e| problem(&e, None))?;
    let r = answer.get().map_err(|e| problem(&e, None))?;
    if !r.get_found() {
        return Ok(None);
    }
    let wire = r.get_definition().map_err(|e| problem(&e, None))?;
    let mut definition = RoleDefinition::new(id, "", "", RoleScope::default());
    definition.names.clear();
    for n in wire.get_names().map_err(|e| problem(&e, None))? {
        let (language, text) = (
            n.get_language()
                .and_then(|t| t.to_string().map_err(Into::into))
                .map_err(|e: capnp::Error| problem(&e, None))?,
            n.get_text()
                .and_then(|t| t.to_string().map_err(Into::into))
                .map_err(|e: capnp::Error| problem(&e, None))?,
        );
        definition.names.insert(language, text);
    }
    // How far the role reaches, so the form can say a missing group before anything is written.
    definition.scope = match wire
        .get_scope()
        .map_err(|e| problem(&capnp::Error::from(e), None))?
    {
        Wire::Personal => RoleScope::Personal,
        Wire::Group => RoleScope::Group,
        Wire::Public => RoleScope::Public,
    };
    Ok(Some(definition))
}

/// Give `person` the role, within the group with this IRI or (`None`) personally. The server judges
/// it (an unknown role, the reach, the kind of card) and says why not.
pub(super) async fn assign(
    roles: &roles::Client,
    person: ObjectId,
    role: &str,
    scope: Option<&str>,
) -> Result<RoleAssignment> {
    let mut request = roles.assign_request();
    {
        let mut params = request.get();
        write_uuid(params.reborrow().init_person(), Some(person.as_uuid()));
        params.reborrow().set_role(role);
        params.reborrow().set_scope(scope.unwrap_or_default());
        params.set_details("{}");
    }
    let answer = request
        .send()
        .promise
        .await
        .map_err(|e| problem(&e, Some(person)))?;
    let a = answer
        .get()
        .and_then(roles::assign_results::Reader::get_assignment)
        .map_err(|e| problem(&e, Some(person)))?;
    let text = |t: capnp::Result<capnp::text::Reader<'_>>| -> Result<String> {
        t.and_then(|t| t.to_string().map_err(Into::into))
            .map_err(|e: capnp::Error| problem(&e, Some(person)))
    };
    let given = text(a.get_scope())?;
    Ok(RoleAssignment {
        id: a
            .get_id()
            .and_then(read_object_id)
            .map_err(|e| problem(&e, Some(person)))?,
        person,
        role: text(a.get_role())?,
        scope: (!given.is_empty()).then_some(given),
        details: serde_json::Map::new(),
    })
}

/// End an assignment.
pub(super) async fn unassign(roles: &roles::Client, assignment: ObjectId) -> Result<()> {
    let mut request = roles.unassign_request();
    write_uuid(request.get().init_assignment(), Some(assignment.as_uuid()));
    request
        .send()
        .promise
        .await
        .map(|_| ())
        .map_err(|e| problem(&e, None))
}
