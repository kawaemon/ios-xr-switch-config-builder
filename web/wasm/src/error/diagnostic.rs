use crate::ast::Span;

/// Represents different kinds of errors that can occur during parsing and validation
#[derive(Debug, Clone)]
pub enum ErrorKind {
    // VLAN-related errors
    VlanNotPresent { vlan: u32, interface: String },
    VlanNotDefinedInDatabase { vlan: u32 },
    VlanNameRequired { vlan: Option<u32> },
    InvalidVlanId { text: String },
    InvalidVlanNumber { text: String },
    InvalidVlanRange { text: String },
    VlanListEmpty,

    // Interface-related errors
    MissingDescription { interface: String },
    InvalidBviNumber { text: String },
    BundledInterfaceCannotConfigureVlans { interface: String, bundle_id: u32 },

    // Switchport mode errors
    UnsupportedSwitchportMode { mode: String },
    AccessModeNotSupported,

    // Trunk action errors
    InvalidTrunkAction { action: String },

    // Generic errors
    Generic { message: String },
}

impl ErrorKind {
    /// Format the error message without span information
    pub fn message(&self) -> String {
        match self {
            ErrorKind::VlanNotPresent { vlan, interface } => {
                format!(
                    "cannot remove VLAN {} from interface {}: VLAN not present in base config",
                    vlan, interface
                )
            }
            ErrorKind::VlanNotDefinedInDatabase { vlan } => {
                format!(
                    "VLAN {} is not defined in vlan database or base config",
                    vlan
                )
            }
            ErrorKind::VlanNameRequired { vlan } => {
                if let Some(v) = vlan {
                    format!("VLAN {} name is required", v)
                } else {
                    "vlan name is required".to_string()
                }
            }
            ErrorKind::InvalidVlanId { text } => {
                format!("invalid vlan id: {}", text)
            }
            ErrorKind::InvalidVlanNumber { text } => {
                format!("invalid vlan number: {}", text)
            }
            ErrorKind::InvalidVlanRange { text } => {
                format!("invalid VLAN range (start must be <= end): {}", text)
            }
            ErrorKind::VlanListEmpty => "vlan list is empty".to_string(),
            ErrorKind::MissingDescription { interface } => {
                format!("interface requires description: {}", interface)
            }
            ErrorKind::InvalidBviNumber { text } => {
                format!("invalid BVI number: {}", text)
            }
            ErrorKind::BundledInterfaceCannotConfigureVlans {
                interface,
                bundle_id,
            } => {
                format!(
                    "interface {} is part of bundle {} and cannot configure VLANs directly. Configure VLANs on Bundle-Ether{} instead",
                    interface, bundle_id, bundle_id
                )
            }
            ErrorKind::UnsupportedSwitchportMode { mode } => {
                format!("switchport mode {} is not supported", mode)
            }
            ErrorKind::AccessModeNotSupported => "switchport access is not supported".to_string(),
            ErrorKind::InvalidTrunkAction { action } => {
                format!("invalid trunk action: {}", action)
            }
            ErrorKind::Generic { message } => message.clone(),
        }
    }
}

/// A diagnostic message with optional span information
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Specific error kind describing the issue.
    pub kind: ErrorKind,
    /// Optional source span for the error.
    pub span: Option<Span>,
}

impl Diagnostic {
    /// Construct a diagnostic without span information.
    pub fn new(kind: ErrorKind) -> Self {
        Diagnostic { kind, span: None }
    }

    /// Construct a diagnostic with an associated source span.
    pub fn with_span(kind: ErrorKind, span: Span) -> Self {
        Diagnostic {
            kind,
            span: Some(span),
        }
    }

    /// Format the diagnostic as a string with optional line number
    /// This maintains compatibility with existing error message format: "message (line N)"
    pub fn format(&self) -> String {
        let message = self.kind.message();
        match self.span {
            Some(span) => format!("{} (line {})", message, span.line.get()),
            None => message,
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl std::error::Error for Diagnostic {}

impl From<String> for Diagnostic {
    fn from(message: String) -> Self {
        Diagnostic::new(ErrorKind::Generic { message })
    }
}

impl From<&str> for Diagnostic {
    fn from(message: &str) -> Self {
        Diagnostic::new(ErrorKind::Generic {
            message: message.to_string(),
        })
    }
}
