//! N6: der Zweck-Katalog als eigener Trait (wie `RetentionSettings`): Verbraucher, `LocalAddressBook`, `GroupsDirectory`
//! und `CompositeDirectory` brauchen ihn nicht. Fähigkeiten (in `CAPABILITIES`): `purpose.read`, `purpose.write`.
//!
//! Pattern: Repository (Trait ohne Standard-Implementierung; Null Object `NullCatalog` in `null.rs`).
//!
//! Rechte und Ebenen (festgeschrieben): Instanz-Zwecke und Instanz-Texte brauchen `purpose.write`. Eigene Person-Zwecke
//! und ein eigener Text der Person zu einem Standard-Zweck (Person-Ebene) legt, ändert und zieht die Person SELBST
//! zurück, ohne Recht. `draft.owner` ungleich dem bisherigen Eigentümer ist `Invalid`; `Person(andere)` ist `Forbidden`.
//! `purposes` liefert je Ebene einen Eintrag (die Person-Überschreibung UND die Instanz-Fassung, damit das Fenster
//! beide zeigt). KI: jede Methode `Forbidden` (kein Orakel).

use super::{Purpose, PurposeDef, PurposeDraft, Reference, Result, Revision};
use async_trait::async_trait;

/// Der Zweck-Katalog einer Instanz.
#[async_trait]
pub trait PurposeCatalog: Send + Sync {
    /// `purpose.read`: erst die 9 Standard-Zwecke in Listen-Reihenfolge, dann Instanz, dann eigene der fragenden Person;
    /// je Ebene (mitgeliefert, Instanz, Person) ein Eintrag.
    ///
    /// # Errors
    /// `Forbidden` für eine KI, `Unavailable`.
    async fn purposes(&self) -> Result<Vec<PurposeDef>>;
    /// `purpose.read`: die Fassung mit dieser Kennung (`PurposeDef::version`, `ConsentEntry::text`): die zugestimmte
    /// Fassung bleibt lesbar (Art. 7 Abs. 1), auch nach `revise_purpose`. Lesen darf, wer `purpose.read` hat oder eine
    /// Einwilligung mit diesem `text` in `consents_of` sieht; sonst `None`. KI: `Forbidden`.
    ///
    /// # Errors
    /// `Forbidden`, `Unavailable`.
    async fn purpose_version(&self, version: &Reference) -> Result<Option<PurposeDef>>;
    /// `purpose.write` (Instance) oder für sich (Person): neuer eigener Zweck `Other(urn:uuid:...)`, `needs_consent` =
    /// true (VORLÄUFIG). Mindestens ein Text, sonst `Invalid`.
    ///
    /// # Errors
    /// `Invalid`, `Forbidden`.
    async fn define_purpose(&self, draft: &PurposeDraft) -> Result<PurposeDef>;
    /// Texte ändern (neue Revision, alte Fassung bleibt lesbar); Standard-Zweck: nur Texte, `None` = erste
    /// Überschreibung dieser Ebene; veraltetes `expected`: `Changed{current}`; Eigentümer wechseln: `Invalid`.
    ///
    /// # Errors
    /// `Changed`, `Invalid`, `Forbidden`.
    async fn revise_purpose(
        &self,
        purpose: &Purpose,
        expected: Option<Revision>,
        draft: &PurposeDraft,
    ) -> Result<PurposeDef>;
    /// Eigenen Zweck zurückziehen (sperrt neues Erfassen: `PurposeRetired`); Standard-Zweck: `StandardPurposeFixed`;
    /// veraltetes `expected`: `Changed`.
    ///
    /// # Errors
    /// `Changed`, `StandardPurposeFixed`, `Forbidden`.
    async fn retire_purpose(&self, purpose: &Purpose, expected: Revision) -> Result<PurposeDef>;
}
