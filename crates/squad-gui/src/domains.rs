pub struct DomainEntry {
    pub value: i64,
    pub label: &'static str,
}

pub struct FieldDomain {
    pub table: &'static str,
    pub column: &'static str,
    pub domain: &'static str,
}

pub const DOMAINS: &[(&str, &[DomainEntry])] = &[
    (
        "Accessory",
        &[
            DomainEntry {
                value: 0,
                label: "None",
            },
            DomainEntry {
                value: 1,
                label: "Unused",
            },
            DomainEntry {
                value: 2,
                label: "Right Bracelet",
            },
            DomainEntry {
                value: 3,
                label: "Left Bracelet",
            },
            DomainEntry {
                value: 4,
                label: "Ankles Tapes",
            },
            DomainEntry {
                value: 5,
                label: "Left Calf Tape",
            },
            DomainEntry {
                value: 6,
                label: "Right Calf Tape",
            },
            DomainEntry {
                value: 7,
                label: "Left Wrist Tape",
            },
            DomainEntry {
                value: 8,
                label: "Right Wrist Tape",
            },
            DomainEntry {
                value: 9,
                label: "Left Finger Tape",
            },
            DomainEntry {
                value: 10,
                label: "Right Finger Tape",
            },
            DomainEntry {
                value: 11,
                label: "Unused",
            },
            DomainEntry {
                value: 12,
                label: "Unused",
            },
            DomainEntry {
                value: 13,
                label: "Left Bracelet",
            },
            DomainEntry {
                value: 14,
                label: "Right Bracelet",
            },
            DomainEntry {
                value: 15,
                label: "Left Wrist Tape",
            },
            DomainEntry {
                value: 16,
                label: "Right Wrist Tape",
            },
            DomainEntry {
                value: 17,
                label: "Unused",
            },
            DomainEntry {
                value: 18,
                label: "Unused",
            },
            DomainEntry {
                value: 19,
                label: "Headbend",
            },
        ],
    ),
    (
        "AccessoryCode",
        &[
            DomainEntry {
                value: 0,
                label: "None",
            },
            DomainEntry {
                value: 1,
                label: "Undefined",
            },
            DomainEntry {
                value: 2,
                label: "Hearphone",
            },
            DomainEntry {
                value: 3,
                label: "Undefined",
            },
            DomainEntry {
                value: 4,
                label: "Left Watch",
            },
            DomainEntry {
                value: 5,
                label: "Right Watch",
            },
            DomainEntry {
                value: 6,
                label: "Left Hand Tape",
            },
            DomainEntry {
                value: 7,
                label: "Right Hand Tape",
            },
            DomainEntry {
                value: 8,
                label: "Left Wristle Tape",
            },
            DomainEntry {
                value: 9,
                label: "Right Wristle Tape",
            },
            DomainEntry {
                value: 10,
                label: "Left Knee Tape",
            },
            DomainEntry {
                value: 11,
                label: "Right Knee Tape",
            },
            DomainEntry {
                value: 12,
                label: "Left Knee Tutor",
            },
            DomainEntry {
                value: 13,
                label: "Right Knee Tutor",
            },
            DomainEntry {
                value: 14,
                label: "Left Ankle Tape",
            },
            DomainEntry {
                value: 15,
                label: "Right Ankle Tape",
            },
            DomainEntry {
                value: 16,
                label: "Gloves",
            },
            DomainEntry {
                value: 17,
                label: "Undefined",
            },
            DomainEntry {
                value: 18,
                label: "Undefined",
            },
            DomainEntry {
                value: 19,
                label: "Undefined",
            },
            DomainEntry {
                value: 20,
                label: "Undefined",
            },
            DomainEntry {
                value: 21,
                label: "Undefined",
            },
            DomainEntry {
                value: 22,
                label: "Left Finger Tape",
            },
            DomainEntry {
                value: 23,
                label: "Right Finger Tape",
            },
            DomainEntry {
                value: 24,
                label: "Left Wristle Wide Tape",
            },
            DomainEntry {
                value: 25,
                label: "Right Wristle Wide Tape",
            },
            DomainEntry {
                value: 26,
                label: "Left Bracelet",
            },
            DomainEntry {
                value: 27,
                label: "Right Bracelet",
            },
        ],
    ),
    (
        "AccessoryColor",
        &[
            DomainEntry {
                value: 0,
                label: "White",
            },
            DomainEntry {
                value: 1,
                label: "Black",
            },
            DomainEntry {
                value: 2,
                label: "Blue",
            },
            DomainEntry {
                value: 3,
                label: "Red",
            },
            DomainEntry {
                value: 4,
                label: "Yellow",
            },
            DomainEntry {
                value: 5,
                label: "Dark Green",
            },
            DomainEntry {
                value: 6,
                label: "Orange",
            },
            DomainEntry {
                value: 7,
                label: "Violet",
            },
            DomainEntry {
                value: 8,
                label: "Brown",
            },
            DomainEntry {
                value: 9,
                label: "Pink",
            },
            DomainEntry {
                value: 10,
                label: "Bordeaux",
            },
            DomainEntry {
                value: 11,
                label: "Cyan",
            },
            DomainEntry {
                value: 12,
                label: "Blue Navy",
            },
            DomainEntry {
                value: 13,
                label: "Undefined",
            },
        ],
    ),
    (
        "AttackTactic1",
        &[
            DomainEntry {
                value: 0,
                label: "Counter Attack",
            },
            DomainEntry {
                value: 1,
                label: "Wing Play",
            },
            DomainEntry {
                value: 2,
                label: "Box Overload",
            },
            DomainEntry {
                value: 3,
                label: "3rd Man Release",
            },
        ],
    ),
    (
        "AttackTactic2",
        &[
            DomainEntry {
                value: 0,
                label: "None",
            },
            DomainEntry {
                value: 1,
                label: "Counter Attack",
            },
            DomainEntry {
                value: 2,
                label: "Wing Play",
            },
            DomainEntry {
                value: 3,
                label: "Box Overload",
            },
            DomainEntry {
                value: 4,
                label: "3rd Man Release",
            },
        ],
    ),
    (
        "BodySizeCode",
        &[
            DomainEntry {
                value: 0,
                label: "Unused",
            },
            DomainEntry {
                value: 1,
                label: "Small",
            },
            DomainEntry {
                value: 2,
                label: "Normal",
            },
            DomainEntry {
                value: 3,
                label: "Big",
            },
            DomainEntry {
                value: 4,
                label: "Unused",
            },
        ],
    ),
    (
        "BodyTypeCode",
        &[
            DomainEntry {
                value: 0,
                label: "Average and Lean",
            },
            DomainEntry {
                value: 1,
                label: "Average",
            },
            DomainEntry {
                value: 2,
                label: "Average and Muscular",
            },
            DomainEntry {
                value: 3,
                label: "Tall and Lean",
            },
            DomainEntry {
                value: 4,
                label: "Tall",
            },
            DomainEntry {
                value: 5,
                label: "Tall and Muscular",
            },
            DomainEntry {
                value: 6,
                label: "Short and Lean",
            },
            DomainEntry {
                value: 7,
                label: "Short",
            },
            DomainEntry {
                value: 8,
                label: "Short and Muscular",
            },
        ],
    ),
    (
        "Confederation",
        &[
            DomainEntry {
                value: 0,
                label: "None",
            },
            DomainEntry {
                value: 1,
                label: "UEFA",
            },
            DomainEntry {
                value: 2,
                label: "CAF",
            },
            DomainEntry {
                value: 3,
                label: "CONMEBOL",
            },
            DomainEntry {
                value: 4,
                label: "AFC",
            },
            DomainEntry {
                value: 5,
                label: "OFC",
            },
            DomainEntry {
                value: 6,
                label: "CONCACAF",
            },
        ],
    ),
    (
        "Continent",
        &[
            DomainEntry {
                value: 0,
                label: "Undefined",
            },
            DomainEntry {
                value: 1,
                label: "Europe",
            },
            DomainEntry {
                value: 2,
                label: "North America",
            },
            DomainEntry {
                value: 3,
                label: "South America",
            },
            DomainEntry {
                value: 4,
                label: "Asia",
            },
            DomainEntry {
                value: 5,
                label: "Africa",
            },
            DomainEntry {
                value: 6,
                label: "Oceania",
            },
        ],
    ),
    (
        "Currency",
        &[
            DomainEntry {
                value: 0,
                label: "€",
            },
            DomainEntry {
                value: 1,
                label: "$",
            },
            DomainEntry {
                value: 2,
                label: "£",
            },
        ],
    ),
    (
        "DefPositioning",
        &[
            DomainEntry {
                value: 0,
                label: "Cover",
            },
            DomainEntry {
                value: 1,
                label: "Offside Trap",
            },
        ],
    ),
    (
        "DefenseTactic1",
        &[
            DomainEntry {
                value: 0,
                label: "Pressing",
            },
            DomainEntry {
                value: 1,
                label: "Offside Trap",
            },
            DomainEntry {
                value: 2,
                label: "Zone Defense",
            },
            DomainEntry {
                value: 3,
                label: "Flat Back",
            },
        ],
    ),
    (
        "DefenseTactic2",
        &[
            DomainEntry {
                value: 0,
                label: "Pressing",
            },
            DomainEntry {
                value: 1,
                label: "Offside Trap",
            },
            DomainEntry {
                value: 2,
                label: "Zone Defense",
            },
            DomainEntry {
                value: 3,
                label: "Flat Back",
            },
            DomainEntry {
                value: 4,
                label: "None",
            },
        ],
    ),
    (
        "DevelopmentType",
        &[
            DomainEntry {
                value: 0,
                label: "Early Bloomer",
            },
            DomainEntry {
                value: 1,
                label: "Regular",
            },
            DomainEntry {
                value: 2,
                label: "Late Bloomer",
            },
        ],
    ),
    (
        "EyeColor",
        &[
            DomainEntry {
                value: 0,
                label: "Blue",
            },
            DomainEntry {
                value: 1,
                label: "Dark Blue",
            },
            DomainEntry {
                value: 2,
                label: "Light Blue",
            },
            DomainEntry {
                value: 3,
                label: "Dark Brown",
            },
            DomainEntry {
                value: 4,
                label: "Brown",
            },
            DomainEntry {
                value: 5,
                label: "Light Brown",
            },
            DomainEntry {
                value: 6,
                label: "Green",
            },
            DomainEntry {
                value: 7,
                label: "Light Green",
            },
        ],
    ),
    (
        "FaceToneCode",
        &[
            DomainEntry {
                value: 0,
                label: "Specific",
            },
            DomainEntry {
                value: 1,
                label: "Light Pink",
            },
            DomainEntry {
                value: 2,
                label: "Pink",
            },
            DomainEntry {
                value: 3,
                label: "Unused",
            },
            DomainEntry {
                value: 4,
                label: "Light Yellow",
            },
            DomainEntry {
                value: 5,
                label: "Medium Yellow",
            },
            DomainEntry {
                value: 6,
                label: "Dark Yellow",
            },
            DomainEntry {
                value: 7,
                label: "Unused",
            },
            DomainEntry {
                value: 8,
                label: "Light Brown",
            },
            DomainEntry {
                value: 9,
                label: "Medium Brown",
            },
            DomainEntry {
                value: 10,
                label: "Dark Brown",
            },
        ],
    ),
    (
        "FacialHairColor",
        &[
            DomainEntry {
                value: 0,
                label: "Black",
            },
            DomainEntry {
                value: 1,
                label: "Blonde",
            },
            DomainEntry {
                value: 2,
                label: "Dark Brown",
            },
            DomainEntry {
                value: 3,
                label: "Light Brown",
            },
            DomainEntry {
                value: 4,
                label: "Red",
            },
        ],
    ),
    (
        "FacialHairStyle",
        &[
            DomainEntry {
                value: 0,
                label: "None",
            },
            DomainEntry {
                value: 1,
                label: "Chin beard",
            },
            DomainEntry {
                value: 2,
                label: "Chin strap",
            },
            DomainEntry {
                value: 3,
                label: "Goatee",
            },
            DomainEntry {
                value: 4,
                label: "Medium Beard",
            },
            DomainEntry {
                value: 5,
                label: "Moustache",
            },
            DomainEntry {
                value: 6,
                label: "Stubble",
            },
            DomainEntry {
                value: 7,
                label: "Tuft",
            },
        ],
    ),
    (
        "HairColor",
        &[
            DomainEntry {
                value: 0,
                label: "Blonde",
            },
            DomainEntry {
                value: 1,
                label: "Black",
            },
            DomainEntry {
                value: 2,
                label: "Dark Blonde",
            },
            DomainEntry {
                value: 3,
                label: "Dark Brown",
            },
            DomainEntry {
                value: 4,
                label: "Light Blonde",
            },
            DomainEntry {
                value: 5,
                label: "Light Brown",
            },
            DomainEntry {
                value: 6,
                label: "Brown",
            },
            DomainEntry {
                value: 7,
                label: "Red",
            },
            DomainEntry {
                value: 8,
                label: "White",
            },
            DomainEntry {
                value: 9,
                label: "Grey",
            },
            DomainEntry {
                value: 10,
                label: "Green",
            },
            DomainEntry {
                value: 11,
                label: "Violet",
            },
        ],
    ),
    (
        "HateLove",
        &[
            DomainEntry {
                value: 0,
                label: "Hate",
            },
            DomainEntry {
                value: 1,
                label: "Love",
            },
        ],
    ),
    (
        "HeadClassCode",
        &[
            DomainEntry {
                value: 0,
                label: "Specific",
            },
            DomainEntry {
                value: 1,
                label: "Generic",
            },
            DomainEntry {
                value: 2,
                label: "Unused",
            },
        ],
    ),
    (
        "InternationalReputation",
        &[
            DomainEntry {
                value: 0,
                label: "Poor",
            },
            DomainEntry {
                value: 1,
                label: "Medium",
            },
            DomainEntry {
                value: 2,
                label: "Good",
            },
            DomainEntry {
                value: 3,
                label: "High",
            },
            DomainEntry {
                value: 4,
                label: "Superstar",
            },
        ],
    ),
    (
        "KitType",
        &[
            DomainEntry {
                value: 0,
                label: "Home Kit",
            },
            DomainEntry {
                value: 1,
                label: "Away Kit",
            },
            DomainEntry {
                value: 2,
                label: "Goalkeeper Kit",
            },
            DomainEntry {
                value: 3,
                label: "Third Kit",
            },
            DomainEntry {
                value: 4,
                label: "Manager Kit",
            },
            DomainEntry {
                value: 5,
                label: "Referee Kit",
            },
            DomainEntry {
                value: 6,
                label: "Training Kit 1",
            },
            DomainEntry {
                value: 7,
                label: "Training Kit 2",
            },
        ],
    ),
    (
        "LegType",
        &[
            DomainEntry {
                value: 0,
                label: "Slim Legs, Low Socks",
            },
            DomainEntry {
                value: 1,
                label: "Slim Legs, Regular Socks",
            },
            DomainEntry {
                value: 2,
                label: "Slim Legs, High Socks",
            },
            DomainEntry {
                value: 3,
                label: "Thick Legs, Low Socks",
            },
            DomainEntry {
                value: 4,
                label: "Thick Legs, Regular Socks",
            },
            DomainEntry {
                value: 5,
                label: "Thick Legs, High Socks",
            },
        ],
    ),
    (
        "LocalizationCode",
        &[
            DomainEntry {
                value: 0,
                label: "Unknown",
            },
            DomainEntry {
                value: 1,
                label: "UK",
            },
            DomainEntry {
                value: 2,
                label: "Spain",
            },
            DomainEntry {
                value: 3,
                label: "Germany",
            },
            DomainEntry {
                value: 4,
                label: "Italy",
            },
            DomainEntry {
                value: 5,
                label: "France",
            },
            DomainEntry {
                value: 6,
                label: "Portugal",
            },
            DomainEntry {
                value: 7,
                label: "Other",
            },
        ],
    ),
    (
        "MowPattern",
        &[
            DomainEntry {
                value: 0,
                label: "Squared",
            },
            DomainEntry {
                value: 1,
                label: "Squared Big",
            },
            DomainEntry {
                value: 2,
                label: "Squared + Circles",
            },
            DomainEntry {
                value: 3,
                label: "Horizontal Big",
            },
            DomainEntry {
                value: 4,
                label: "Horizontal + Left Diagonal",
            },
            DomainEntry {
                value: 5,
                label: "Horizontal + Right Diagonal",
            },
            DomainEntry {
                value: 6,
                label: "Squared + Diagonal Box",
            },
            DomainEntry {
                value: 7,
                label: "Squared + Diagonal Box",
            },
            DomainEntry {
                value: 8,
                label: "Diagonal Small",
            },
            DomainEntry {
                value: 9,
                label: "Squared + Circles",
            },
            DomainEntry {
                value: 10,
                label: "Five Circles",
            },
            DomainEntry {
                value: 11,
                label: "Kilt Style",
            },
            DomainEntry {
                value: 12,
                label: "Horizontal",
            },
            DomainEntry {
                value: 13,
                label: "Diagonal",
            },
        ],
    ),
    (
        "NetColor",
        &[
            DomainEntry {
                value: 0,
                label: "Small Square White",
            },
            DomainEntry {
                value: 1,
                label: "Big Square Red",
            },
            DomainEntry {
                value: 2,
                label: "Small Square Green",
            },
            DomainEntry {
                value: 3,
                label: "Big Square Yellow",
            },
            DomainEntry {
                value: 4,
                label: "Big Square Yellow",
            },
            DomainEntry {
                value: 5,
                label: "Big Square Yellow",
            },
            DomainEntry {
                value: 6,
                label: "Hexagon White",
            },
            DomainEntry {
                value: 7,
                label: "Hexagon Red",
            },
            DomainEntry {
                value: 8,
                label: "Hexagon Blue",
            },
            DomainEntry {
                value: 9,
                label: "Hexagon Yellow",
            },
            DomainEntry {
                value: 10,
                label: "Hexagon White",
            },
        ],
    ),
    (
        "NoTopBottom",
        &[
            DomainEntry {
                value: 0,
                label: "No",
            },
            DomainEntry {
                value: 1,
                label: "Top",
            },
            DomainEntry {
                value: 2,
                label: "Bottom",
            },
        ],
    ),
    (
        "NoYes",
        &[
            DomainEntry {
                value: 0,
                label: "No",
            },
            DomainEntry {
                value: 1,
                label: "Yes",
            },
        ],
    ),
    (
        "PlayingStyle",
        &[
            DomainEntry {
                value: 0,
                label: "Shot Stopper",
            },
            DomainEntry {
                value: 1,
                label: "Attacking Winger",
            },
            DomainEntry {
                value: 2,
                label: "Wing Backs",
            },
            DomainEntry {
                value: 3,
                label: "Stopper",
            },
            DomainEntry {
                value: 4,
                label: "Sweeper",
            },
            DomainEntry {
                value: 5,
                label: "Ball Winner",
            },
            DomainEntry {
                value: 6,
                label: "Box to Box",
            },
            DomainEntry {
                value: 7,
                label: "Defensive Midfielder",
            },
            DomainEntry {
                value: 8,
                label: "Midfield Maestro",
            },
            DomainEntry {
                value: 9,
                label: "Dribbler",
            },
            DomainEntry {
                value: 10,
                label: "Counter Attacker",
            },
            DomainEntry {
                value: 11,
                label: "Clinical Striker",
            },
            DomainEntry {
                value: 12,
                label: "Target Man",
            },
            DomainEntry {
                value: 13,
                label: "Unused",
            },
        ],
    ),
    (
        "PositionAll",
        &[
            DomainEntry {
                value: 0,
                label: "Goalkeeper",
            },
            DomainEntry {
                value: 1,
                label: "Sweeper",
            },
            DomainEntry {
                value: 2,
                label: "R.Wing Back",
            },
            DomainEntry {
                value: 3,
                label: "R. Back",
            },
            DomainEntry {
                value: 4,
                label: "R.C. Back",
            },
            DomainEntry {
                value: 5,
                label: "C. Back",
            },
            DomainEntry {
                value: 6,
                label: "L.C. Back",
            },
            DomainEntry {
                value: 7,
                label: "L. Back",
            },
            DomainEntry {
                value: 8,
                label: "L.Wing Back",
            },
            DomainEntry {
                value: 9,
                label: "R. Def. Midfielder",
            },
            DomainEntry {
                value: 10,
                label: "C. Def. Midfielder",
            },
            DomainEntry {
                value: 11,
                label: "L. Def. Midfielder",
            },
            DomainEntry {
                value: 12,
                label: "R. Midfielder",
            },
            DomainEntry {
                value: 13,
                label: "R.C. Midfielder",
            },
            DomainEntry {
                value: 14,
                label: "C. Midfielder",
            },
            DomainEntry {
                value: 15,
                label: "L.C. Midfielder",
            },
            DomainEntry {
                value: 16,
                label: "L. Midfielder",
            },
            DomainEntry {
                value: 17,
                label: "R. Adv. Midfielder",
            },
            DomainEntry {
                value: 18,
                label: "C. Adv. Midfielder",
            },
            DomainEntry {
                value: 19,
                label: "L. Adv. Midfielder",
            },
            DomainEntry {
                value: 20,
                label: "R. Forfward",
            },
            DomainEntry {
                value: 21,
                label: "C. Forfward",
            },
            DomainEntry {
                value: 22,
                label: "L. Forfward",
            },
            DomainEntry {
                value: 23,
                label: "R. Wing",
            },
            DomainEntry {
                value: 24,
                label: "R. Striker",
            },
            DomainEntry {
                value: 25,
                label: "C. Striker",
            },
            DomainEntry {
                value: 26,
                label: "L. Striker",
            },
            DomainEntry {
                value: 27,
                label: "L. Wing",
            },
            DomainEntry {
                value: 28,
                label: "Substitute",
            },
            DomainEntry {
                value: 29,
                label: "Tribune",
            },
        ],
    ),
    (
        "PositionAlternative",
        &[
            DomainEntry {
                value: 0,
                label: "None",
            },
            DomainEntry {
                value: 1,
                label: "Goalkeeper",
            },
            DomainEntry {
                value: 2,
                label: "Sweeper",
            },
            DomainEntry {
                value: 3,
                label: "R.Wing Back",
            },
            DomainEntry {
                value: 4,
                label: "R. Back",
            },
            DomainEntry {
                value: 5,
                label: "R.C. Back",
            },
            DomainEntry {
                value: 6,
                label: "C. Back",
            },
            DomainEntry {
                value: 7,
                label: "L.C. Back",
            },
            DomainEntry {
                value: 8,
                label: "L. Back",
            },
            DomainEntry {
                value: 9,
                label: "L.Wing Back",
            },
            DomainEntry {
                value: 10,
                label: "R. Def. Midfielder",
            },
            DomainEntry {
                value: 11,
                label: "C. Def. Midfielder",
            },
            DomainEntry {
                value: 12,
                label: "L. Def. Midfielder",
            },
            DomainEntry {
                value: 13,
                label: "R. Midfielder",
            },
            DomainEntry {
                value: 14,
                label: "R.C. Midfielder",
            },
            DomainEntry {
                value: 15,
                label: "C. Midfielder",
            },
            DomainEntry {
                value: 16,
                label: "L.C. Midfielder",
            },
            DomainEntry {
                value: 17,
                label: "L. Midfielder",
            },
            DomainEntry {
                value: 18,
                label: "R. Adv. Midfielder",
            },
            DomainEntry {
                value: 19,
                label: "C. Adv. Midfielder",
            },
            DomainEntry {
                value: 20,
                label: "L. Adv. Midfielder",
            },
            DomainEntry {
                value: 21,
                label: "R. Forfward",
            },
            DomainEntry {
                value: 22,
                label: "C. Forfward",
            },
            DomainEntry {
                value: 23,
                label: "L. Forfward",
            },
            DomainEntry {
                value: 24,
                label: "R. Wing",
            },
            DomainEntry {
                value: 25,
                label: "R. Striker",
            },
            DomainEntry {
                value: 26,
                label: "C. Striker",
            },
            DomainEntry {
                value: 27,
                label: "L. Striker",
            },
            DomainEntry {
                value: 28,
                label: "L. Wing",
            },
            DomainEntry {
                value: 29,
                label: "Substitute",
            },
            DomainEntry {
                value: 30,
                label: "Tribune",
            },
        ],
    ),
    (
        "PositionMovement",
        &[
            DomainEntry {
                value: 0,
                label: "Sweeper",
            },
            DomainEntry {
                value: 1,
                label: "R.Wing Back",
            },
            DomainEntry {
                value: 2,
                label: "R. Back",
            },
            DomainEntry {
                value: 3,
                label: "R.C. Back",
            },
            DomainEntry {
                value: 4,
                label: "C. Back",
            },
            DomainEntry {
                value: 5,
                label: "L.C. Back",
            },
            DomainEntry {
                value: 6,
                label: "L. Back",
            },
            DomainEntry {
                value: 7,
                label: "L.Wing Back",
            },
            DomainEntry {
                value: 8,
                label: "R. Def. Midfielder",
            },
            DomainEntry {
                value: 9,
                label: "C. Def. Midfielder",
            },
            DomainEntry {
                value: 10,
                label: "L. Def. Midfielder",
            },
            DomainEntry {
                value: 11,
                label: "R. Midfielder",
            },
            DomainEntry {
                value: 12,
                label: "R.C. Midfielder",
            },
            DomainEntry {
                value: 13,
                label: "C. Midfielder",
            },
            DomainEntry {
                value: 14,
                label: "L.C. Midfielder",
            },
            DomainEntry {
                value: 15,
                label: "L. Midfielder",
            },
            DomainEntry {
                value: 16,
                label: "R. Adv. Midfielder",
            },
            DomainEntry {
                value: 17,
                label: "C. Adv. Midfielder",
            },
            DomainEntry {
                value: 18,
                label: "L. Adv. Midfielder",
            },
            DomainEntry {
                value: 19,
                label: "R. Forfward",
            },
            DomainEntry {
                value: 20,
                label: "C. Forfward",
            },
            DomainEntry {
                value: 21,
                label: "L. Forfward",
            },
            DomainEntry {
                value: 22,
                label: "R. Wing",
            },
            DomainEntry {
                value: 23,
                label: "R. Striker",
            },
            DomainEntry {
                value: 24,
                label: "C. Striker",
            },
            DomainEntry {
                value: 25,
                label: "L. Striker",
            },
            DomainEntry {
                value: 26,
                label: "L. Wing",
            },
            DomainEntry {
                value: 27,
                label: "Substitute",
            },
            DomainEntry {
                value: 28,
                label: "Tribune",
            },
        ],
    ),
    (
        "PositionReserve",
        &[
            DomainEntry {
                value: 0,
                label: "Substitute",
            },
            DomainEntry {
                value: 1,
                label: "Tribune",
            },
        ],
    ),
    (
        "Positioning",
        &[
            DomainEntry {
                value: 0,
                label: "Organized",
            },
            DomainEntry {
                value: 1,
                label: "Free Form",
            },
        ],
    ),
    (
        "Regions",
        &[
            DomainEntry {
                value: 0,
                label: "United Kingdom",
            },
            DomainEntry {
                value: 1,
                label: "Western Europe",
            },
            DomainEntry {
                value: 2,
                label: "Southern Europe",
            },
            DomainEntry {
                value: 3,
                label: "Central Europe",
            },
            DomainEntry {
                value: 4,
                label: "The Americas",
            },
            DomainEntry {
                value: 5,
                label: "Scandinavia",
            },
            DomainEntry {
                value: 6,
                label: "Rest of World",
            },
            DomainEntry {
                value: 7,
                label: "Endurance",
            },
            DomainEntry {
                value: 8,
                label: "Master",
            },
        ],
    ),
    (
        "RightLeft",
        &[
            DomainEntry {
                value: 0,
                label: "Right",
            },
            DomainEntry {
                value: 1,
                label: "Left",
            },
        ],
    ),
    (
        "Rivaltype",
        &[
            DomainEntry {
                value: 0,
                label: "Historical",
            },
            DomainEntry {
                value: 1,
                label: "Derby",
            },
            DomainEntry {
                value: 2,
                label: "Regional",
            },
        ],
    ),
    (
        "SeatColor",
        &[
            DomainEntry {
                value: 0,
                label: "Dark Red",
            },
            DomainEntry {
                value: 1,
                label: "Dark Grey",
            },
            DomainEntry {
                value: 2,
                label: "Green",
            },
            DomainEntry {
                value: 3,
                label: "Dark Blue",
            },
            DomainEntry {
                value: 4,
                label: "Light Blue",
            },
            DomainEntry {
                value: 5,
                label: "Light Red",
            },
            DomainEntry {
                value: 6,
                label: "Dark Yellow",
            },
            DomainEntry {
                value: 7,
                label: "Light Grey",
            },
        ],
    ),
    (
        "ShortLong",
        &[
            DomainEntry {
                value: 0,
                label: "Short",
            },
            DomainEntry {
                value: 1,
                label: "Long",
            },
        ],
    ),
    (
        "SideBurns",
        &[
            DomainEntry {
                value: 0,
                label: "None",
            },
            DomainEntry {
                value: 1,
                label: "Short",
            },
            DomainEntry {
                value: 2,
                label: "Long",
            },
            DomainEntry {
                value: 3,
                label: "Unused",
            },
        ],
    ),
    (
        "SkinToneCode",
        &[
            DomainEntry {
                value: 0,
                label: "Light Pink",
            },
            DomainEntry {
                value: 1,
                label: "Pink",
            },
            DomainEntry {
                value: 2,
                label: "Dark Pink",
            },
            DomainEntry {
                value: 3,
                label: "Light Yellow",
            },
            DomainEntry {
                value: 4,
                label: "Medium Yellow",
            },
            DomainEntry {
                value: 5,
                label: "Dark Yellow",
            },
            DomainEntry {
                value: 6,
                label: "Very Light Brown",
            },
            DomainEntry {
                value: 7,
                label: "Light Brown",
            },
            DomainEntry {
                value: 8,
                label: "Medium Brown",
            },
            DomainEntry {
                value: 9,
                label: "Dark Brown",
            },
        ],
    ),
    (
        "SkinTypeCode",
        &[
            DomainEntry {
                value: 0,
                label: "Clean",
            },
            DomainEntry {
                value: 1,
                label: "Freckled",
            },
            DomainEntry {
                value: 2,
                label: "Rough",
            },
        ],
    ),
    (
        "Speech",
        &[
            DomainEntry {
                value: 0,
                label: "Enghish",
            },
            DomainEntry {
                value: 1,
                label: "French",
            },
            DomainEntry {
                value: 2,
                label: "German",
            },
            DomainEntry {
                value: 3,
                label: "Italian",
            },
            DomainEntry {
                value: 4,
                label: "Spanish",
            },
            DomainEntry {
                value: 5,
                label: "Brazilian",
            },
            DomainEntry {
                value: 6,
                label: "",
            },
            DomainEntry {
                value: 7,
                label: "Korean",
            },
            DomainEntry {
                value: 8,
                label: "Dutch",
            },
            DomainEntry {
                value: 9,
                label: "Denish",
            },
            DomainEntry {
                value: 10,
                label: "Swedish",
            },
            DomainEntry {
                value: 11,
                label: "Norwegian",
            },
            DomainEntry {
                value: 12,
                label: "Portuguese",
            },
            DomainEntry {
                value: 13,
                label: "Other",
            },
        ],
    ),
    (
        "StadiumCondition",
        &[
            DomainEntry {
                value: 0,
                label: "Damaged",
            },
            DomainEntry {
                value: 1,
                label: "Medium",
            },
            DomainEntry {
                value: 2,
                label: "Good",
            },
            DomainEntry {
                value: 3,
                label: "Unused",
            },
        ],
    ),
    (
        "StadiumType",
        &[
            DomainEntry {
                value: 0,
                label: "Normal",
            },
            DomainEntry {
                value: 1,
                label: "Training",
            },
        ],
    ),
    (
        "TeamMentality",
        &[
            DomainEntry {
                value: 0,
                label: "Defensive",
            },
            DomainEntry {
                value: 1,
                label: "Neutral",
            },
            DomainEntry {
                value: 2,
                label: "Offensive",
            },
        ],
    ),
    (
        "TicketLvl",
        &[
            DomainEntry {
                value: 0,
                label: "Low Price",
            },
            DomainEntry {
                value: 1,
                label: "Medium Price",
            },
            DomainEntry {
                value: 2,
                label: "High Price",
            },
        ],
    ),
    (
        "TimeOfDay",
        &[
            DomainEntry {
                value: 0,
                label: "Cloudy day",
            },
            DomainEntry {
                value: 1,
                label: "Clear day",
            },
            DomainEntry {
                value: 2,
                label: "Unused",
            },
            DomainEntry {
                value: 3,
                label: "Night",
            },
            DomainEntry {
                value: 4,
                label: "Sunset",
            },
            DomainEntry {
                value: 5,
                label: "Not assigned",
            },
        ],
    ),
    (
        "TimeOfDayWeather",
        &[
            DomainEntry {
                value: 0,
                label: "Dry",
            },
            DomainEntry {
                value: 1,
                label: "Dry or Rain",
            },
            DomainEntry {
                value: 2,
                label: "Dry or Snow",
            },
            DomainEntry {
                value: 3,
                label: "Dry or Rain or Snow",
            },
        ],
    ),
    (
        "TournamentId",
        &[
            DomainEntry {
                value: 0,
                label: "National Champion",
            },
            DomainEntry {
                value: 1,
                label: "National Cup",
            },
            DomainEntry {
                value: 2,
                label: "To be found",
            },
            DomainEntry {
                value: 3,
                label: "To be found",
            },
            DomainEntry {
                value: 4,
                label: "UEFA Cup",
            },
            DomainEntry {
                value: 5,
                label: "Champions League",
            },
            DomainEntry {
                value: 6,
                label: "Toyota Cup",
            },
        ],
    ),
    (
        "TraitTwo",
        &[
            DomainEntry {
                value: 0,
                label: "None",
            },
            DomainEntry {
                value: 1,
                label: "Cross Claimer (Goalkeeper)",
            },
            DomainEntry {
                value: 2,
                label: "Rush Out (Goalkeeper)",
            },
            DomainEntry {
                value: 3,
                label: "Cross Claimer+Rush Out (Goalkeeper)",
            },
            DomainEntry {
                value: 4,
                label: "None",
            },
            DomainEntry {
                value: 5,
                label: "None",
            },
            DomainEntry {
                value: 6,
                label: "None",
            },
            DomainEntry {
                value: 7,
                label: "None",
            },
            DomainEntry {
                value: 8,
                label: "None",
            },
            DomainEntry {
                value: 9,
                label: "None",
            },
            DomainEntry {
                value: 10,
                label: "None",
            },
            DomainEntry {
                value: 11,
                label: "None",
            },
            DomainEntry {
                value: 12,
                label: "None",
            },
            DomainEntry {
                value: 13,
                label: "None",
            },
            DomainEntry {
                value: 14,
                label: "None",
            },
            DomainEntry {
                value: 15,
                label: "None",
            },
            DomainEntry {
                value: 16,
                label: "Falls if pushed (Player hidden trait)",
            },
        ],
    ),
    (
        "Weather",
        &[
            DomainEntry {
                value: 0,
                label: "Sunny",
            },
            DomainEntry {
                value: 1,
                label: "Mostly Sunny",
            },
            DomainEntry {
                value: 2,
                label: "Mostly Rainy",
            },
            DomainEntry {
                value: 3,
                label: "Rainy",
            },
            DomainEntry {
                value: 4,
                label: "Heavy Rain",
            },
            DomainEntry {
                value: 5,
                label: "Unused",
            },
        ],
    ),
];

pub const FIELD_DOMAINS: &[FieldDomain] = &[
    FieldDomain {
        table: "*",
        column: "position0",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position1",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position2",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position3",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position4",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position5",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position6",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position7",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position8",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position9",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "position10",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "*",
        column: "skintonecode",
        domain: "SkinToneCode",
    },
    FieldDomain {
        table: "*",
        column: "accessorycolour",
        domain: "AccessoryColor",
    },
    FieldDomain {
        table: "*",
        column: "accessorycode",
        domain: "AccessoryCode",
    },
    FieldDomain {
        table: "players",
        column: "preferredposition1",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "players",
        column: "preferredposition2",
        domain: "PositionAlternative",
    },
    FieldDomain {
        table: "players",
        column: "preferredposition3",
        domain: "PositionAlternative",
    },
    FieldDomain {
        table: "players",
        column: "preferredposition4",
        domain: "PositionAlternative",
    },
    FieldDomain {
        table: "teamplayerlinks",
        column: "position",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "nations",
        column: "confederation",
        domain: "Confederation",
    },
    FieldDomain {
        table: "teamplayerlinks",
        column: "transferdone",
        domain: "NoYes",
    },
    FieldDomain {
        table: "player_grudgelove",
        column: "love_or_hate",
        domain: "HateLove",
    },
    FieldDomain {
        table: "referee",
        column: "haircolorcode",
        domain: "HairColor",
    },
    FieldDomain {
        table: "referee",
        column: "facialhairtypecode",
        domain: "FacialHairStyle",
    },
    FieldDomain {
        table: "referee",
        column: "eyecolorcode",
        domain: "EyeColor",
    },
    FieldDomain {
        table: "players",
        column: "eyecolorcode",
        domain: "EyeColor",
    },
    FieldDomain {
        table: "team_manager",
        column: "ticket_lvl",
        domain: "TicketLvl",
    },
    FieldDomain {
        table: "career_transfer_list",
        column: "is_loan",
        domain: "NoYes",
    },
    FieldDomain {
        table: "career_transfer_list",
        column: "sold",
        domain: "NoYes",
    },
    FieldDomain {
        table: "team_manager",
        column: "ticket_lvl",
        domain: "TicketLvl",
    },
    FieldDomain {
        table: "stadiums",
        column: "timeofday1",
        domain: "TimeOfDay",
    },
    FieldDomain {
        table: "stadiums",
        column: "timeofday2",
        domain: "TimeOfDay",
    },
    FieldDomain {
        table: "stadiums",
        column: "timeofday3",
        domain: "TimeOfDay",
    },
    FieldDomain {
        table: "stadiums",
        column: "timeofday4",
        domain: "TimeOfDay",
    },
    FieldDomain {
        table: "stadiums",
        column: "tod1weather",
        domain: "TimeOfDayWeather",
    },
    FieldDomain {
        table: "stadiums",
        column: "tod2weather",
        domain: "TimeOfDayWeather",
    },
    FieldDomain {
        table: "stadiums",
        column: "tod3weather",
        domain: "TimeOfDayWeather",
    },
    FieldDomain {
        table: "stadiums",
        column: "tod4weather",
        domain: "TimeOfDayWeather",
    },
    FieldDomain {
        table: "players",
        column: "preferredfoot",
        domain: "RightLeft",
    },
    FieldDomain {
        table: "teams",
        column: "speechcountryid",
        domain: "Speech",
    },
    FieldDomain {
        table: "formations",
        column: "position1",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position2",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position3",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position4",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position5",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position6",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position7",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position8",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position9",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formations",
        column: "position10",
        domain: "PositionMovement",
    },
    FieldDomain {
        table: "formupdate",
        column: "position",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "players",
        column: "sleevelength",
        domain: "ShortLong",
    },
    FieldDomain {
        table: "players",
        column: "jerseysleevelengthcode",
        domain: "ShortLong",
    },
    FieldDomain {
        table: "referees",
        column: "jerseysleevelengthcode",
        domain: "ShortLong",
    },
    FieldDomain {
        table: "referees",
        column: "sleevelength",
        domain: "ShortLong",
    },
    FieldDomain {
        table: "referees",
        column: "isinternationalreferee",
        domain: "NoYes",
    },
    FieldDomain {
        table: "teams",
        column: "genericbanner",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "seatcolor",
        domain: "SeatColor",
    },
    FieldDomain {
        table: "stadiums",
        column: "track",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "netcolor",
        domain: "NetColor",
    },
    FieldDomain {
        table: "stadiums",
        column: "shadowleft",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "shadowright",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "hasnighttime",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "stadiumtype",
        domain: "StadiumType",
    },
    FieldDomain {
        table: "stadiums",
        column: "hasovercast",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "hasclearday",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "hassunset",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "hasretractableroof",
        domain: "NoYes",
    },
    FieldDomain {
        table: "stadiums",
        column: "localizationcode",
        domain: "LocalizationCode",
    },
    FieldDomain {
        table: "stadiums",
        column: "mowpattern",
        domain: "MowPattern",
    },
    FieldDomain {
        table: "stadiums",
        column: "stadiumcondition",
        domain: "StadiumCondition",
    },
    FieldDomain {
        table: "teamkits",
        column: "teamkittypetechid",
        domain: "KitType",
    },
    FieldDomain {
        table: "teamkits",
        column: "nameplacement",
        domain: "NoTopBottom",
    },
    FieldDomain {
        table: "teamkits",
        column: "numberplacementback",
        domain: "NoYes",
    },
    FieldDomain {
        table: "teamkits",
        column: "numberplacementfront",
        domain: "NoYes",
    },
    FieldDomain {
        table: "teamkits",
        column: "shortnumberplacement",
        domain: "NoYes",
    },
    FieldDomain {
        table: "country",
        column: "continentid",
        domain: "Continent",
    },
    FieldDomain {
        table: "players",
        column: "facialhaircolorcode",
        domain: "FacialHairColor",
    },
    FieldDomain {
        table: "referees",
        column: "facialhaircolorcode",
        domain: "FacialHairColor",
    },
    FieldDomain {
        table: "players",
        column: "haircolorcode",
        domain: "HairColor",
    },
    FieldDomain {
        table: "challenge_regions",
        column: "id",
        domain: "Regions",
    },
    FieldDomain {
        table: "challenge_countries",
        column: "region",
        domain: "Regions",
    },
    FieldDomain {
        table: "challenges",
        column: "region",
        domain: "Regions",
    },
    FieldDomain {
        table: "challenge_leagues",
        column: "region",
        domain: "Regions",
    },
    FieldDomain {
        table: "players",
        column: "facialhairtypecode",
        domain: "FacialHairStyle",
    },
    FieldDomain {
        table: "players",
        column: "sideburnscode",
        domain: "SideBurns",
    },
    FieldDomain {
        table: "referees",
        column: "sideburnscode",
        domain: "SideBurns",
    },
    FieldDomain {
        table: "players",
        column: "facetonecode",
        domain: "FaceToneCode",
    },
    FieldDomain {
        table: "referees",
        column: "facetonecode",
        domain: "FaceToneCode",
    },
    FieldDomain {
        table: "players",
        column: "trait2",
        domain: "TraitTwo",
    },
    FieldDomain {
        table: "players",
        column: "legtypeid",
        domain: "LegType",
    },
    FieldDomain {
        table: "referees",
        column: "legtypeid",
        domain: "LegType",
    },
    FieldDomain {
        table: "players",
        column: "playingstyle",
        domain: "PlayingStyle",
    },
    FieldDomain {
        table: "players",
        column: "developmenttypecode",
        domain: "DevelopmentType",
    },
    FieldDomain {
        table: "players",
        column: "accessoryid1",
        domain: "Accessory",
    },
    FieldDomain {
        table: "players",
        column: "accessoryid2",
        domain: "Accessory",
    },
    FieldDomain {
        table: "players",
        column: "accessoryid3",
        domain: "Accessory",
    },
    FieldDomain {
        table: "players",
        column: "accessoryid4",
        domain: "Accessory",
    },
    FieldDomain {
        table: "players",
        column: "accessoryid5",
        domain: "Accessory",
    },
    FieldDomain {
        table: "players",
        column: "injuryprone",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "flexibility",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "adaptability",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "ambition",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "comesforcrosses",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "diver",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "divesintotackles",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "earlycrosser",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "holdsup",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "latecrosser",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "leadership",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "longshottaker",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "longthrows",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "lowconcentration",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "onetimepasser",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "playmaker",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "puncher",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "pushesupforcorners",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "rushesoutofgoal",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "selfish",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "staysongoalline",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "technicaldribbler",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "highclubidentification",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "lowclubidentification",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "mediadarling",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "fansfavourite",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "scandalprone",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "hightrainingworkrate",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "lowtrainingworkrate",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "inflexible",
        domain: "NoYes",
    },
    FieldDomain {
        table: "players",
        column: "bodysizecode",
        domain: "BodySizeCode",
    },
    FieldDomain {
        table: "referees",
        column: "bodysizecode",
        domain: "BodySizeCode",
    },
    FieldDomain {
        table: "players",
        column: "bodytypecode",
        domain: "BodyTypeCode",
    },
    FieldDomain {
        table: "referees",
        column: "bodytypecode",
        domain: "BodyTypeCode",
    },
    FieldDomain {
        table: "players",
        column: "headclasscode",
        domain: "HeadClassCode",
    },
    FieldDomain {
        table: "referees",
        column: "headclasscode",
        domain: "HeadClassCode",
    },
    FieldDomain {
        table: "rivals",
        column: "rivaltype",
        domain: "Rivaltype",
    },
    FieldDomain {
        table: "players",
        column: "skintypecode",
        domain: "SkinTypeCode",
    },
    FieldDomain {
        table: "referees",
        column: "skintypecode",
        domain: "SkinTypeCode",
    },
    FieldDomain {
        table: "players",
        column: "internationalreputation",
        domain: "InternationalReputation",
    },
    FieldDomain {
        table: "teamwrite",
        column: "teammentality",
        domain: "TeamMentality",
    },
    FieldDomain {
        table: "teamwrite",
        column: "attacktactic1",
        domain: "AttackTactic1",
    },
    FieldDomain {
        table: "teamwrite",
        column: "attacktactic2",
        domain: "AttackTactic2",
    },
    FieldDomain {
        table: "teamwrite",
        column: "defensetactic1",
        domain: "DefenseTactic1",
    },
    FieldDomain {
        table: "teamwrite",
        column: "defensetactic2",
        domain: "DefenseTactic2",
    },
    FieldDomain {
        table: "tournamentvictories",
        column: "tournamentid",
        domain: "TournamentId",
    },
    FieldDomain {
        table: "fieldpositionboundingboxes",
        column: "positionid",
        domain: "PositionAll",
    },
    FieldDomain {
        table: "teams",
        column: "buspositioning",
        domain: "Positioning",
    },
    FieldDomain {
        table: "teams",
        column: "ccpositioning",
        domain: "Positioning",
    },
    FieldDomain {
        table: "teams",
        column: "defdefenderline",
        domain: "DefPositioning",
    },
    FieldDomain {
        table: "weather",
        column: "m0",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m1",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m2",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m3",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m4",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m5",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m6",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m7",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m8",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m9",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m10",
        domain: "Weather",
    },
    FieldDomain {
        table: "weather",
        column: "m11",
        domain: "Weather",
    },
];

pub fn domain_values(domain: &str) -> Option<&'static [DomainEntry]> {
    DOMAINS
        .iter()
        .find(|(name, _)| *name == domain)
        .map(|(_, entries)| *entries)
}

pub fn field_domain(table_name: &str, column: &str) -> Option<&'static str> {
    FIELD_DOMAINS
        .iter()
        .find(|fd| fd.table == table_name && fd.column == column)
        .or_else(|| {
            FIELD_DOMAINS
                .iter()
                .find(|fd| fd.table == "*" && fd.column == column)
        })
        .map(|fd| fd.domain)
}

pub fn label_for(table_name: &str, column: &str, value: i64) -> Option<&'static str> {
    let domain = field_domain(table_name, column)?;
    let entries = domain_values(domain)?;
    entries.iter().find(|e| e.value == value).map(|e| e.label)
}
