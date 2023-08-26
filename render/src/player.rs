pub const COLORS: [&'static str; 8] = [
    "#0433FF", "#00A0DF", "#37BB00", "#FFBE0E", "#E16200", "#C11A00", "#C12EBF", "#6127C4",
];

pub struct Player<'a> {
    pub id: i32,
    pub color: &'static str,
    pub fleets: Vec<&'a api::order::Fleet>,
}
