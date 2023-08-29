/// https://forum.ironhelmet.com/t/api-documentation-player-written/7533

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct ID(
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub  i32,
);

impl From<i32> for ID {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<ID> for i32 {
    fn from(value: ID) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderRequest {
    #[serde(rename = "type")]
    pub ty: std::borrow::Cow<'static, str>,
    pub order: std::borrow::Cow<'static, str>,
    pub version: Option<i32>,
    pub game_number: i64,
}

impl OrderRequest {
    pub fn full_universe_report(game_number: i64) -> Self {
        Self {
            ty: "order".into(),
            order: "full_universe_report".into(),
            version: None,
            game_number,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderResponse {
    pub event: String,
    pub report: Report,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub fleets: std::collections::HashMap<ID, Fleet>,
    pub fleet_speed: f32,
    pub paused: bool,
    pub productions: i32,
    pub tick_fragment: i32,
    pub now: i64,
    pub tick_rate: i32,
    pub production_rate: i32,
    pub stars: std::collections::HashMap<ID, Star>,
    pub stars_for_victory: i32,
    pub game_over: i32,
    pub started: bool,
    pub start_time: i64,
    pub total_stars: i32,
    pub production_counter: i32,
    pub trade_scanned: i32,
    pub tick: i32,
    pub trade_cost: i32,
    pub name: String,
    pub player_uid: i32,
    pub admin: i32,
    pub turn_based: i32,
    pub war: i32,
    pub players: std::collections::HashMap<ID, Player>,
    pub turn_based_time_out: i64,
}

#[derive(
    Debug, Clone, Copy, PartialEq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr,
)]
#[repr(i32)]
pub enum FleetOrderType {
    DoNothing = 0,
    CollectAll = 1,
    DropAll = 2,
    Collect = 3,
    Drop = 4,
    CollectAllBut = 5,
    DropAllBut = 6,
    GarrisonStar = 7,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct FleetOrder((i32, i32, FleetOrderType, i32));

impl FleetOrder {
    pub fn delay(&self) -> i32 {
        self.0 .0
    }

    pub fn target_star_uid(&self) -> i32 {
        self.0 .1
    }

    pub fn fleet_order_type(&self) -> FleetOrderType {
        self.0 .2
    }

    pub fn fleet_order_amount(&self) -> i32 {
        self.0 .3
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fleet {
    /// Unique ID for the carrier’s current star
    pub ouid: Option<i32>,
    pub uid: i32,
    /// Looping, 1 = looped, 0 = not looped
    pub l: i32,
    pub o: Vec<FleetOrder>,
    pub n: String,
    pub puid: i32,
    pub w: i32,
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub y: f32,
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub x: f32,
    /// Number of ships (strength)
    pub st: i32,
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub lx: f32,
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub ly: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Star {
    /// Unique ID for the star (matches to the key in the parent object)
    pub uid: i32,
    /// The current name of the star
    pub n: String,
    /// Player ID of the player who owns the star
    pub puid: i32,
    /// Flag for if the star is visible. 0 = no, 1 = yes
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub v: i32,
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub y: f32,
    #[serde(deserialize_with = "serde_aux::field_attributes::deserialize_number_from_string")]
    pub x: f32,
    #[serde(flatten)]
    pub extra: Option<VisibleStar>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisibleStar {
    /// Where ships/tick is not a whole number, the amount currently produced
    pub c: f32,
    /// Current level of economy
    pub e: i32,
    /// Current level of industry
    pub i: i32,
    /// Current level of science
    pub s: i32,
    /// Resource level of the star (including terraforming bonus)
    pub r: i32,
    /// The presence of a warpgate. 0= no gate, 1 = gate
    pub ga: i32,
    /// Natural resources of the star
    pub nr: i32,
    /// Number of ships on the star
    pub st: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TechnologyExtra {
    pub sv: f32,
    pub research: i32,
    pub bv: f32,
    pub brr: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Technology {
    pub value: f32,
    pub level: i32,
    #[serde(flatten)]
    pub extra: Option<TechnologyExtra>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Technologies {
    pub scanning: Technology,
    pub propulsion: Technology,
    pub terraforming: Technology,
    pub research: Technology,
    pub weapons: Technology,
    pub banking: Technology,
    pub manufacturing: Technology,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MyPlayer {
    pub researching: String,
    /// Unique ID for the player. Matches the object key
    pub uid: i32,
    pub ai: i32,
    /// Unique ID for the player’s home star
    pub huid: i32,
    pub total_fleets: i32,
    pub ready: i32,
    pub karma_to_give: i32,
    pub war: std::collections::HashMap<ID, i32>,
    pub total_industry: i32,
    pub total_stars: i32,
    pub regard: i32,
    pub conceded: i32,
    pub total_science: i32,
    pub stars_abandoned: i32,
    pub cash: i32,
    pub total_strength: i32,
    pub alias: String,
    pub tech: Technologies,
    pub avatar: i32,
    pub researching_next: String,
    pub total_economy: i32,
    pub countdown_to_war: std::collections::HashMap<ID, i32>,
    pub missed_turns: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OtherPlayer {
    pub total_industry: i32,
    pub regard: i32,
    pub total_science: i32,
    pub uid: i32,
    pub ai: i32,
    pub huid: i32,
    pub total_stars: i32,
    pub total_fleets: i32,
    pub total_strength: i32,
    pub alias: String,
    pub tech: Technologies,
    pub avatar: i32,
    pub conceded: i32,
    pub ready: i32,
    pub total_economy: i32,
    pub missed_turns: i32,
    pub karma_to_give: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Player {
    Mine(MyPlayer),
    Theirs(OtherPlayer),
}
