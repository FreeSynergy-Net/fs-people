# people_v2.capnp: PROBE copy of fs-interfaces v0.0.27 people.capnp plus the person_directory_v2 delta.
# APPEND ONLY: the delta is the line `directory @5` in `interface People` (marked) and everything after the line
# `# ---- person_directory_v2 delta ----` at the end. `diff` against v0.0.27 shows only additions.
# fs-interfaces itself is NOT changed by this probe.
# fs-people contract — DRAFT 0.1 (2026-11-06)
#
# STATUS: DRAFT. Owner program: fs-people. Lives in fs-interfaces (ADR 0045).
# Consumers so far: none (0 of 2). Provider so far: fs-people-core (store-backed).
#
# One record per person, several roles, no inheritance, no duplicates (ADR 0052).
#   Relation = what someone is to me: JSContact `relatedTo` / vCard RELATED. Part of the card.
#   Role     = what someone is within a group or organisation: org:Role, org:Membership. NOT part of
#              the card, so an export of a card never reveals a role (ADR 0029).
# fs-people holds the person; the owner of a role holds that role's fields. Role definitions are
# objects loaded as configuration; there is no fixed list of roles in any program.
#
# AIs (ADR 0054): a marked participant = JSContact `kind` "application" (RFC 9553 §2.1.4) plus the
# role "ai", because plain "application" also covers software that is no AI. Nothing restricts them.
# Naming the operator is planned, its form is open; `RoleAssignment.details` leaves room for it.
# Locking AIs out is not part of this draft.
#
# A person travels as JSContact JSON text, which stays authoritative (ADR 0014, 0050), next to the
# AS2 envelope. Evolution rules (Cap'n Proto): never renumber or remove fields/methods; only append.

@0xf41de01e6495cb24;

using C = import "common.capnp";
using Uuid = C.Uuid;
using Cursor = C.Cursor;
using Envelope = C.Envelope;
using SupportReport = C.SupportReport;
using T = import "/types.capnp";
using ChangeListener = C.ChangeListener;
using Subscription = C.Subscription;

struct Person {
  envelope  @0 :Envelope;              # AS2 `Profile` envelope (id, iri, name, context "people")
  jscontact @1 :Text;                  # the JSContact `Card` as JSON (authoritative content)
  roles     @2 :List(RoleAssignment);  # only what the caller may see (ADR 0029)
  revision  @3 :UInt64;                # changes at every write of the card (for updateIf); compare for equality only, it is no counter
}

# one page of people in name order
struct PersonPage { items @0 :List(Person); next @1 :Cursor; }

# all members optional; empty = no restriction; combined with AND
struct PersonFilter {
  kind @0 :Text;                       # JSContact kind
  role @1 :Text;                       # full IRI of the role
  term @2 :Text;                       # substring of name, nickname, e-mail, phone, organisation
}

struct ImportReport {
  created @0 :List(Uuid);
  skipped @1 :List(Text);              # uids that already existed
  failed  @2 :List(Text);
  unclear @3 :List(Text);              # stored, with a sensitive marking that names no category (ADR 0029)
}

struct PersonProblem {
  path @0 :Text;                       # JSON pointer into the card: /emails/e1
  code @1 :Text;                       # what is wrong, as a code
}

# ---- Relations: what someone is to me (part of the card) ----

struct Relation {
  uid      @0 :Text;                   # uid of the related card
  relation @1 :Text;                   # RFC 9553 §2.1.8 type, e.g. "friend", or <domain>:<value>
}

# ---- Same person: two cards stay, a link connects them (link, don't merge) ----

enum SameState { open @0; confirmed @1; rejected @2; }

struct SamePersonProposal {
  a       @0 :Uuid;
  b       @1 :Uuid;
  reasons @2 :List(Text);
}

# ---- Roles: objects, ours and yours defined the same way (ADR 0052) ----

struct Localized { language @0 :Text; text @1 :Text; }

struct RoleField {
  name        @0 :Text;
  standard    @1 :Text;                # the standard it follows; empty = none
  description @2 :List(Localized);
}

enum RoleScope { personal @0; group @1; public @2; }

struct RoleDefinition {
  id          @0 :Text;                # full IRI: the file's base plus the short id
  names       @1 :List(Localized);
  description @2 :List(Localized);
  standard    @3 :Text;                # the standard it follows; empty = none (a gap)
  fields      @4 :List(RoleField);     # held by the owner, not by fs-people
  owner       @5 :Text;                # the program responsible for it
  scope       @6 :RoleScope;
  cardKind    @7 :Text;                # cards with this role must have this kind; empty = any
  type        @8 :Text;                # `org:Role` or `skos:Concept`; empty = `skos:Concept` (`org:Role` is a subclass of it, org 5.3.5)
  broader     @9 :List(Text);          # full IRIs of more general roles (skos:broader); may point into another file
  closeMatch  @10 :List(Text);         # full IRIs of the same idea elsewhere (skos:closeMatch); may point into another file
  standardRefs @11 :List(Text);        # addresses of the standard's terms (rdfs:seeAlso)
}

struct RoleAssignment {
  id      @0 :Uuid;
  person  @1 :Uuid;
  role    @2 :Text;                    # full IRI of the role
  scope   @3 :Text;                    # IRI of the group; empty = personal
  details @4 :Text;                    # JSON object; lossless; for roles owned by fs-people
}

# ---- The interface; capability ids in comments ----

interface People {
  supportReport @0 () -> (report :SupportReport);
  persons       @1 () -> (persons :Persons);
  roles         @2 () -> (roles :Roles);
  subscribe     @3 (listener :ChangeListener) -> (subscription :Subscription);   # person.subscribe
  # The data types this program owns (`Person`) and what can be done with them (ADR 0066), for a generated frontend.
  types         @4 () -> (catalog :T.TypeCatalog);
  # person_directory_v2 (PROBE): the consumer side of People. Appended; no existing ordinal changes.
  directory     @5 () -> (directory :Directory);
}

interface Persons {
  create         @0  (jscontact :Text) -> (person :Person);                      # person.create
  update         @1  (id :Uuid, jscontact :Text) -> (person :Person);            # person.update
  delete         @2  (id :Uuid) -> ();                                           # person.delete
  get            @3  (id :Uuid) -> (found :Bool, person :Person);                # person.read
  getByUid       @4  (uid :Text) -> (found :Bool, person :Person);               # person.read
  list           @5  (filter :PersonFilter, cursor :Cursor) -> (page :PersonPage); # person.read
  findByDescribes @6 (iri :Text) -> (items :List(Person));                       # person.find
  findByEmail    @7  (email :Text) -> (items :List(Person));                     # person.find
  findByPhone    @8  (phone :Text) -> (items :List(Person));                     # person.find
  importText     @9  (mediaType :Text, text :Text) -> (report :ImportReport);    # person.import
  # A sensitive card, or a sensitive member of a list, goes along only if its uid is in `released`
  # (ADR 0029). A refused card is a problem; members left out of a list are named in `withheld`.
  exportText     @10 (id :Uuid, mediaType :Text, released :List(Text)) -> (text :Text, withheld :List(Text)); # person.export
  relate         @11 (id :Uuid, otherUid :Text, relation :Text) -> (person :Person);   # relation.write
  unrelate       @12 (id :Uuid, otherUid :Text, relation :Text) -> (person :Person);   # relation.write
  relations      @13 (id :Uuid) -> (items :List(Relation));                      # relation.read
  proposals      @14 () -> (items :List(SamePersonProposal));                    # same.propose
  confirmSame    @15 (a :Uuid, b :Uuid) -> (link :Uuid);                         # same.link
  unlinkSame     @16 (link :Uuid) -> ();                                         # same.link
  linkedTo       @17 (id :Uuid) -> (ids :List(Uuid));                            # same.read
  # JSContact group cards as lists of people. Rights and belonging are fs-groups (ADR 0050).
  createList     @18 (name :Text) -> (person :Person);                           # cardgroup.write
  addToList      @19 (list :Uuid, member :Uuid) -> (person :Person);             # cardgroup.write
  removeFromList @20 (list :Uuid, member :Uuid) -> (person :Person);             # cardgroup.write
  listMembers    @21 (list :Uuid) -> (items :List(Person));                      # cardgroup.read
  listsOf        @22 (member :Uuid) -> (items :List(Person));                    # cardgroup.read
  updateIf       @23 (id :Uuid, revision :UInt64, jscontact :Text) -> (person :Person);  # person.update, only if the person still has `revision`; otherwise it fails with a reason starting `conflict:` and writes nothing (compare-and-set; atomic inside one provider process only)
  # ADR 0058. A pair nobody answered is `open`; `confirmSame` makes it `confirmed`; `rejectSame` makes it
  # `rejected` and it is not proposed again; unlinking a confirmed pair makes it `rejected`;
  # `reopenSame` takes a rejection back. Linking copies nothing from the person.
  rejectSame     @24 (a :Uuid, b :Uuid) -> (rejection :Uuid);                    # same.link
  reopenSame     @25 (rejection :Uuid) -> ();                                    # same.link
  sameState      @26 (a :Uuid, b :Uuid) -> (state :SameState);                   # same.read
  # Commands of the type description `Person` (ADR 0066).
  # `patch`: a PatchObject (RFC 8984 section 1.4.9) as JSON text: path (a JSON pointer without its first `/`) to value,
  # `null` removes. All or nothing; the result is validated like `update`. A path through what is not there is refused.
  # DEPRECATED for frontends (rule of 2026-11-10: no JSON in a form): a frontend uses `change` (typed, field by field). `patch`
  # stays (never remove) as the means of tools that already hold a PatchObject.
  patch          @27 (id :Uuid, patch :Text) -> (person :Person);               # person.update
  # `check`: the card against RFC 9553 and the rules of the basic types. A problem is a path and a code
  # (translation key suffix `problem.<code>`), never a sentence; empty = in order. It changes nothing.
  check          @28 (id :Uuid) -> (problems :List(PersonProblem));             # person.check
  # `suggest`: completions for what is typed in a field (`field` is the field name of the description, e.g. `emails`).
  suggest        @29 (field :Text, typed :Text) -> (suggestions :List(Text));    # person.suggest
  # `change`: the command `change` of the description `Person`: an `Edit` (types.capnp), field path plus typed values, field by
  # field. A field not in the `Edit` stays; one with no values is cleared; otherwise the values replace the field's. The
  # entries of a multiple field that the description does not show (their sub-objects' contexts, pref, label) stay with the
  # values that remain. The backend turns it into the card's PatchObject. All or nothing, validated like `update`.
  change         @30 (id :Uuid, edit :T.Edit) -> (person :Person);              # person.update
}

interface Roles {
  define     @0 (definition :RoleDefinition) -> ();                              # role.define
  undefine   @1 (id :Text) -> ();                                                # role.define
  loadConfig @2 (json :Text) -> ();                                              # role.configure: checked, then swapped at once
  get        @3 (id :Text) -> (found :Bool, definition :RoleDefinition);         # role.read
  list       @4 () -> (items :List(RoleDefinition));                             # role.read
  assign     @5 (person :Uuid, role :Text, scope :Text, details :Text) -> (assignment :RoleAssignment); # role.assign
  unassign   @6 (assignment :Uuid) -> ();                                        # role.assign
  rolesOf    @7 (person :Uuid) -> (items :List(RoleAssignment));                 # role.read
  peopleWith @8 (role :Text, scope :Text) -> (ids :List(Uuid));                  # role.read
  markAi     @9 (person :Uuid) -> (assignment :RoleAssignment);                  # ai.mark
  isAi       @10 (person :Uuid) -> (ai :Bool);                                   # ai.mark
}

# ---- person_directory_v2 delta (PROBE, 2026-11-17; Entwurf PERSON-DIRECTORY-V2-ENTWURF.md) ----
# Wire form of the trait `PersonDirectory` v2. A time is `C.Timestamp` (UTC millis) plus an optional zone (normalisation rule:
# time = UTC + zone; a date stays a date). Errors travel as `C.Problem` (RFC 9457), never as a sentence.

enum PurposeKind {                                   # order = PURPOSES in fs-people-core
  contract @0; legalObligation @1; ownUse @2; photo @3; publication @4; newsletter @5;
  aiAssistance @6; disclosureToThirdParty @7; other @8;
}
struct Purpose { kind @0 :PurposeKind; other @1 :Text; }          # `other`: IRI (urn:uuid:... for an own purpose), only for kind other

struct When { union { at @0 :C.Timestamp; on @1 :Text; } }          # `on`: a date YYYY-MM-DD
struct PersonLink { id @0 :Uuid; art @1 :Text; }                    # PersonRef: id plus art (person, profile, group)

enum ConsentOriginKind { byPerson @0; form @1; import @2; oral @3; erasure @4; situationEnded @5; }
enum LifetimeKind { whileScope @0; until @1; }
struct Lifetime { kind @0 :LifetimeKind; until @1 :C.Timestamp; } # `until` only for kind until

enum RecordingKind { photo @0; audio @1; video @2; transcript @3; location @4; }
enum RecordingKeepKind { days @0; untilSituationEnds @1; untilDeletedByHand @2; }
struct RecordingNotice { kind @0 :RecordingKind; keep @1 :RecordingKeepKind; days @2 :UInt32; text @3 :Text; }  # moves to fs-types with its next tag

struct ConsentEntry {
  person @0 :PersonLink; givenBy @1 :PersonLink; hasGivenBy @2 :Bool;
  purpose @3 :Purpose; scope @4 :Text; recipient @5 :PersonLink; hasRecipient @6 :Bool;
  given @7 :When; origin @8 :ConsentOriginKind; evidence @9 :Text; text @10 :Text;
  lifetime @11 :Lifetime; hasLifetime @12 :Bool;
  notice @13 :RecordingNotice; hasNotice @14 :Bool;
}
struct Withdrawal { at @0 :When; zone @1 :Text; origin @2 :ConsentOriginKind; evidence @3 :Text; }   # zone: IANA id, empty = instance zone
struct Consent { id @0 :Uuid; rev @1 :UInt64; entry @2 :ConsentEntry; recordedBy @3 :PersonLink; recordedAt @4 :C.Timestamp;
  withdrawn @5 :Withdrawal; hasWithdrawn @6 :Bool; retainUntil @7 :Text; }   # retainUntil: a date, empty = active
struct ConsentQuery { person @0 :PersonLink; purpose @1 :Purpose; scope @2 :Text; recipient @3 :PersonLink; hasRecipient @4 :Bool; }

enum ExcerptFieldKind { fullName @0; givenName @1; surname @2; organization @3; postal @4; email @5; phone @6; birthday @7; birthMonthDay @8; }
enum WithheldWhy { notShared @0; unverifiable @1; }
struct Withheld { field @0 :ExcerptFieldKind; why @1 :WithheldWhy; }
struct ShareUse { field @0 :ExcerptFieldKind; share @1 :Uuid; }
struct ExcerptValue { field @0 :ExcerptFieldKind; text @1 :Text; }        # typed text of the field's basic type
struct ExcerptRequest { fields @0 :List(ExcerptFieldKind); purpose @1 :Purpose; scope @2 :Text; recipient @3 :PersonLink; hasRecipient @4 :Bool; }
struct Excerpt { person @0 :PersonLink; source @1 :Text; captured @2 :C.Timestamp; purpose @3 :Purpose; consent @4 :Uuid; hasConsent @5 :Bool;
  values @6 :List(ExcerptValue); missing @7 :List(ExcerptFieldKind); withheld @8 :List(Withheld); shares @9 :List(ShareUse); }

enum PurposeOwnerKind { instance @0; person @1; }
struct PurposeText { language @0 :Text; label @1 :Localized; explanation @2 :Localized; }
struct PurposeDraft { ownerKind @0 :PurposeOwnerKind; owner @1 :PersonLink; texts @2 :List(PurposeText); closeMatch @3 :Text; recording @4 :RecordingKind; hasRecording @5 :Bool; }
struct PurposeDef { purpose @0 :Purpose; ownerKind @1 :PurposeOwnerKind; owner @2 :PersonLink; builtIn @3 :Bool; rev @4 :UInt64; version @5 :Text;
  texts @6 :List(PurposeText); needsConsent @7 :Bool; closeMatch @8 :Text; retired @9 :C.Timestamp; hasRetired @10 :Bool; }

# The 11 capabilities are named per method: person.read person.find person.pick person.propose person.excerpt consent.read consent.check
# consent.write (8 of v1) plus consent.situation purpose.read purpose.write. Resolve/suggest/propose/changes of v1 are carried by the
# existing `Persons` interface in the real wire step; the probe lists the v2 delta only.
interface Directory {
  excerpt        @0 (person :PersonLink, request :ExcerptRequest) -> (excerpt :Excerpt);                 # person.excerpt
  consentsOf     @1 (person :PersonLink) -> (items :List(Consent));                                      # consent.read
  checkConsent   @2 (query :ConsentQuery) -> (found :Bool, id :Uuid);                                    # consent.check
  recordConsent  @3 (entry :ConsentEntry) -> (consent :Consent);                                         # consent.write
  withdrawConsent @4 (id :Uuid, withdrawal :Withdrawal) -> (consent :Consent);                           # consent.write
  endSituation   @5 (scope :Text, at :When, evidence :Text) -> (ids :List(Uuid));                        # consent.situation (programme accounts only)
  purposes       @6 () -> (items :List(PurposeDef));                                                     # purpose.read
  definePurpose  @7 (draft :PurposeDraft) -> (def :PurposeDef);                                          # purpose.write (Instance) / own (Person)
  revisePurpose  @8 (purpose :Purpose, hasExpected :Bool, expected :UInt64, draft :PurposeDraft) -> (def :PurposeDef);  # purpose.write
  retirePurpose  @9 (purpose :Purpose, expected :UInt64) -> (def :PurposeDef);                           # purpose.write
}
