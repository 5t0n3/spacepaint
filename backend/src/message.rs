use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PNGFile(pub Vec<u8>);

#[derive(Serialize, Deserialize)]
pub enum ModificationType {
    Heat,
    Cool,
    Humidify,
    Dehumidify,
    Wind,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct LatLong {
    pub lat: f64,
    pub long: f64,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Rect {
    pub top_left: LatLong,
    pub bottom_right: LatLong,
}

#[derive(Serialize, Deserialize)]
pub enum Packet {
    AssignId {
        client_id: u64,
    },
    Snapshot {
        data: PNGFile,
        location: Rect,
    },
    Modification {
        tpe: ModificationType,
        points: Vec<LatLong>,
        brush_size_degrees: f64,
        client_id: u64,
    },
    Viewport {
        area: Rect,
        client_id: u64,
    },
}

pub fn serialize_packet(payload: Packet) -> Result<Vec<u8>> {
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    payload.serialize(&mut serializer)?;

    Ok(serializer.view().to_vec())
}
