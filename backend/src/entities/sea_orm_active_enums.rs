//! Hand-authored string-backed enums shared by the generated entities.
//! Kept in this module so `sea-orm-cli generate entity` never overwrites them.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[sea_orm(string_value = "student")]
    Student,
    #[sea_orm(string_value = "faculty")]
    Faculty,
    #[sea_orm(string_value = "admin")]
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    #[sea_orm(string_value = "door")]
    Door,
    #[sea_orm(string_value = "box")]
    Box,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentStatus {
    #[sea_orm(string_value = "pending_code")]
    PendingCode,
    #[sea_orm(string_value = "awaiting_scan")]
    AwaitingScan,
    #[sea_orm(string_value = "bound")]
    Bound,
    #[sea_orm(string_value = "expired")]
    Expired,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "expired")]
    Expired,
    #[sea_orm(string_value = "consumed")]
    Consumed,
    #[sea_orm(string_value = "closed")]
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum EventDecision {
    #[sea_orm(string_value = "granted")]
    Granted,
    #[sea_orm(string_value = "denied")]
    Denied,
    #[sea_orm(string_value = "info")]
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum AccessEventType {
    #[sea_orm(string_value = "door_granted")]
    DoorGranted,
    #[sea_orm(string_value = "door_denied_unregistered")]
    DoorDeniedUnregistered,
    #[sea_orm(string_value = "door_denied_inactive")]
    DoorDeniedInactive,
    #[sea_orm(string_value = "box_granted")]
    BoxGranted,
    #[sea_orm(string_value = "box_denied_no_session")]
    BoxDeniedNoSession,
    #[sea_orm(string_value = "box_denied_unregistered")]
    BoxDeniedUnregistered,
    #[sea_orm(string_value = "box_denied_session_expired")]
    BoxDeniedSessionExpired,
    #[sea_orm(string_value = "enroll_code_issued")]
    EnrollCodeIssued,
    #[sea_orm(string_value = "enroll_bound")]
    EnrollBound,
    #[sea_orm(string_value = "enroll_failed")]
    EnrollFailed,
    /// Someone tried to record a checkout with a code the cabinet did not issue
    /// to them, or one already spent.
    #[sea_orm(string_value = "checkout_code_rejected")]
    CheckoutCodeRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum SmsStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "sent")]
    Sent,
    #[sea_orm(string_value = "failed")]
    Failed,
}
