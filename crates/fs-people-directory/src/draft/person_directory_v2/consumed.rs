//! Der Port zur Freigabe (fs-access, Rechte-Teil-Trait "Freigabe"): People FRAGT nur, ob die beschriebene Person ein
//! sensibles Feld der fragenden Person freigegeben hat (N2, Antwort 41); die Verwaltung der Freigaben gehört fs-access.
//! People fragt als Programm-Konto mit dem Recht `share.check`.
//!
//! Pattern: Port (Anbieter: fs-access; im Einzelplatz der Null-Port, der nie eine Freigabe kennt).

use super::{ObjectId, PersonRef, Unreachable};
use async_trait::async_trait;

/// Stufe der Geburtstags-Freigabe (Antwort 2: zweistufig).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BirthdayPrecision {
    /// "ohne Jahr": Monat und Tag.
    MonthDay,
    /// "mit Jahr": das volle Datum.
    Full,
}

/// Feld, das freigegeben werden kann. `non_exhaustive`: neue sensible Felder kommen additiv.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ShareField {
    Birthday(BirthdayPrecision),
    Postal,
    Email,
    Phone,
}

/// Port: aktive Freigaben von Feldern einer Person.
#[async_trait]
pub trait FieldShares: Send + Sync {
    /// Aktive Freigabe von `owner` für `field` an `viewer` (wirksame Gruppen, Ablauf, Sperre schlägt Erlaubnis: alles
    /// fs-access); `None` = keine. Die Stufe `Full` deckt auch `MonthDay`, nicht umgekehrt (K3).
    ///
    /// # Errors
    /// `Unreachable`, wenn fs-access nicht antwortet; der Aufrufer behandelt das fail-closed (`Unverifiable`, K5).
    async fn field_share(
        &self,
        owner: &PersonRef,
        field: ShareField,
        viewer: &PersonRef,
    ) -> Result<Option<ObjectId>, Unreachable>;
}

/// Null-Port (Einzelplatz, kein fs-access): kennt nie eine Freigabe.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoFieldShares;

#[async_trait]
impl FieldShares for NoFieldShares {
    async fn field_share(
        &self,
        _owner: &PersonRef,
        _field: ShareField,
        _viewer: &PersonRef,
    ) -> Result<Option<ObjectId>, Unreachable> {
        Ok(None)
    }
}
