#[derive(Debug)]
pub enum BinaryField {
    Uuid(&'static str),
    U8(&'static str),
    U32(&'static str),
    I64(&'static str),
    F64(&'static str),
    LengthPrefixed {
        length_type: LengthType,
        name: &'static str,
    },
}


#[derive(Debug)]
pub enum LengthType {
    U8,
    U16,
    U32,
}

impl LengthType {
    pub fn byte_len(&self) -> usize {
        match self {
            LengthType::U8 => 1,
            LengthType::U16 => 2,
            LengthType::U32 => 4,
        }
    }
}

#[derive(Debug)]
pub struct BinaryLayout {
    pub name: &'static str,
    pub fields: Vec<BinaryField>,
}

pub fn account_layout() -> BinaryLayout {
    BinaryLayout {
        name: "Account",
        fields: vec![
            BinaryField::Uuid("id"),
            BinaryField::LengthPrefixed {
                length_type: LengthType::U8,
                name: "name",
            },
            BinaryField::U8("account_type"),
            BinaryField::I64("created_at"),
            BinaryField::LengthPrefixed {
                length_type: LengthType::U8,
                name: "system_id",
            },
        ],
    }
}

pub fn transaction_layout() -> BinaryLayout {
    BinaryLayout {
        name: "Transaction",
        fields: vec![
            BinaryField::Uuid("id"),
            BinaryField::LengthPrefixed {
                length_type: LengthType::U8,
                name: "description",
            },
            BinaryField::LengthPrefixed {
                length_type: LengthType::U32,
                name: "metadata",
            },
            BinaryField::I64("timestamp"),
        ],
    }
}

pub fn entry_layout() -> BinaryLayout {
    BinaryLayout {
        name: "Entry",
        fields: vec![
            BinaryField::Uuid("id"),
            BinaryField::Uuid("transaction_id"),
            BinaryField::Uuid("account_id"),
            BinaryField::F64("amount"),
        ],
    }
}

pub fn system_layout() -> BinaryLayout {
    BinaryLayout {
        name: "System",
        fields: vec![
            BinaryField::LengthPrefixed {
                length_type: LengthType::U8,
                name: "system_id",
            },
            BinaryField::LengthPrefixed {
                length_type: LengthType::U8,
                name: "description",
            },
        ],
    }
}

pub fn conversion_graph_layout() -> BinaryLayout {
    BinaryLayout {
        name: "ConversionGraph",
        fields: vec![
            BinaryField::LengthPrefixed {
                length_type: LengthType::U8,
                name: "graph",
            },
            BinaryField::F64("rate"), // rate
            BinaryField::I64("rate_since"), // rate_since
        ],
    }
}


pub fn all_layouts() -> Vec<BinaryLayout> {
    vec![
        account_layout(),
        transaction_layout(),
        entry_layout(),
        system_layout(),
        conversion_graph_layout(),
    ]
}
    
