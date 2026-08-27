//! Built-in source and target adapters.
//!
//! These cover every system shown in the "Spaghetti Processing" diagram.
//! Each adapter is a thin, named stub that satisfies the connector traits.
//! Real production adapters are implemented by inheriting the trait and
//! providing actual I/O — the Fabric never changes when you do that.

use crate::connector::{
    DataBatch, DeliveryReceipt, ExtractionCursor, SourceConnector, SourceId, TargetConnector,
    TargetId,
};
use crate::contract::{FieldSpec, FieldType, SchemaContract};
use crate::exception::FabricException;

// ── Source Adapters ───────────────────────────────────────────────────────────

macro_rules! declare_source {
    ($name:ident, $id:literal, $display:literal, $contract:expr) => {
        pub struct $name;

        impl SourceConnector for $name {
            fn source_id(&self) -> SourceId {
                SourceId($id)
            }
            fn display_name(&self) -> &'static str {
                $display
            }
            fn schema(&self) -> SchemaContract {
                $contract
            }

            fn extract(&self, _cursor: &ExtractionCursor) -> Result<DataBatch, FabricException> {
                // Production: replace with real I/O (HTTP, JDBC, file parse, etc.)
                Ok(DataBatch::new(SourceId($id), vec![], 0))
            }
        }
    };
}

declare_source!(
    ErpConnector,
    "erp.sovereign",
    "ERP System",
    SchemaContract::new(
        "erp.invoice",
        1,
        vec![
            FieldSpec::required("erp_id", 0x1001, FieldType::Integer),
            FieldSpec::required("amount", 0x1002, FieldType::Decimal),
            FieldSpec::required("currency", 0x1003, FieldType::Text),
            FieldSpec::optional("description", 0x1004, FieldType::Text),
            FieldSpec::optional("created_at", 0x1005, FieldType::Timestamp),
        ]
    )
);

declare_source!(
    CrmConnector,
    "crm.sovereign",
    "CRM System",
    SchemaContract::new(
        "crm.contact",
        1,
        vec![
            FieldSpec::required("contact_id", 0x2001, FieldType::Integer),
            FieldSpec::required("full_name", 0x2002, FieldType::Text),
            FieldSpec::required("email", 0x2003, FieldType::Text),
            FieldSpec::optional(
                "phone",
                0x2004,
                FieldType::Nullable(Box::new(FieldType::Text))
            ),
            FieldSpec::optional("segment", 0x2005, FieldType::Text),
        ]
    )
);

declare_source!(
    HrSystemConnector,
    "hr.sovereign",
    "HR System",
    SchemaContract::new(
        "hr.employee",
        1,
        vec![
            FieldSpec::required("employee_id", 0x3001, FieldType::Integer),
            FieldSpec::required("name", 0x3002, FieldType::Text),
            FieldSpec::required("department", 0x3003, FieldType::Text),
            FieldSpec::optional("hire_date", 0x3004, FieldType::Timestamp),
            FieldSpec::optional("grade", 0x3005, FieldType::Text),
        ]
    )
);

declare_source!(
    LegacySystemConnector,
    "legacy.sovereign",
    "Legacy System",
    SchemaContract::new(
        "legacy.record",
        1,
        vec![
            FieldSpec::required("rec_id", 0x4001, FieldType::Integer),
            FieldSpec::required("payload", 0x4002, FieldType::Bytes),
            FieldSpec::optional("source_code", 0x4003, FieldType::Text),
        ]
    )
);

declare_source!(
    ExternalPartnerConnector,
    "partner.sovereign",
    "External Partner",
    SchemaContract::new(
        "partner.transaction",
        1,
        vec![
            FieldSpec::required("txn_id", 0x5001, FieldType::Text),
            FieldSpec::required("amount", 0x5002, FieldType::Decimal),
            FieldSpec::required("partner_ref", 0x5003, FieldType::Text),
            FieldSpec::optional("timestamp", 0x5004, FieldType::Timestamp),
        ]
    )
);

declare_source!(
    ExcelFileConnector,
    "excel.sovereign",
    "Excel Files",
    SchemaContract::new(
        "excel.row",
        1,
        vec![
            FieldSpec::required("row_index", 0x6001, FieldType::Integer),
            FieldSpec::required("raw_columns", 0x6002, FieldType::Bytes),
            FieldSpec::optional("sheet_name", 0x6003, FieldType::Text),
            FieldSpec::optional("file_path", 0x6004, FieldType::Text),
        ]
    )
);

declare_source!(
    EmailInboxConnector,
    "email.sovereign",
    "Email / Inbox",
    SchemaContract::new(
        "email.message",
        1,
        vec![
            FieldSpec::required("message_id", 0x7001, FieldType::Text),
            FieldSpec::required("subject", 0x7002, FieldType::Text),
            FieldSpec::required("sender", 0x7003, FieldType::Text),
            FieldSpec::optional("body", 0x7004, FieldType::Text),
            FieldSpec::optional("received_at", 0x7005, FieldType::Timestamp),
        ]
    )
);

declare_source!(
    ThirdPartyApiConnector,
    "api.sovereign",
    "Third-Party API",
    SchemaContract::new(
        "api.event",
        1,
        vec![
            FieldSpec::required("event_id", 0x8001, FieldType::Text),
            FieldSpec::required("event_type", 0x8002, FieldType::Text),
            FieldSpec::required("payload", 0x8003, FieldType::Bytes),
            FieldSpec::optional("api_version", 0x8004, FieldType::Text),
            FieldSpec::optional("emitted_at", 0x8005, FieldType::Timestamp),
        ]
    )
);

// ── Target Adapters ───────────────────────────────────────────────────────────

macro_rules! declare_target {
    ($name:ident, $id:literal, $display:literal, $contract:expr) => {
        pub struct $name;

        impl TargetConnector for $name {
            fn target_id(&self) -> TargetId {
                TargetId($id)
            }
            fn display_name(&self) -> &'static str {
                $display
            }
            fn schema_expectation(&self) -> SchemaContract {
                $contract
            }

            fn deliver(&self, batch: DataBatch) -> Result<DeliveryReceipt, FabricException> {
                // Production: replace with real I/O (SQL INSERT, REST POST, file write, etc.)
                Ok(DeliveryReceipt {
                    target_id: TargetId($id),
                    records_accepted: batch.len(),
                    token: b"ack".to_vec(),
                })
            }
        }
    };
}

declare_target!(
    DataWarehouseTarget,
    "dw.central",
    "Data Warehouse",
    SchemaContract::new("dw.canonical", 1, vec![])
);

declare_target!(
    ReportingToolsTarget,
    "reporting.tools",
    "Reporting Tools",
    SchemaContract::new("reporting.feed", 1, vec![])
);

declare_target!(
    DashboardTarget,
    "dashboard.sovereign",
    "Dashboards",
    SchemaContract::new("dashboard.metric", 1, vec![])
);

declare_target!(
    OtherApplicationsTarget,
    "apps.sovereign",
    "Other Applications",
    SchemaContract::new("apps.event", 1, vec![])
);

declare_target!(
    FileExportTarget,
    "file.export",
    "File Exports",
    SchemaContract::new("file.export.record", 1, vec![])
);

declare_target!(
    ExternalPortalTarget,
    "portal.sovereign",
    "External Portals",
    SchemaContract::new("portal.payload", 1, vec![])
);

declare_target!(
    NotificationTarget,
    "notify.sovereign",
    "Notifications",
    SchemaContract::new("notify.message", 1, vec![])
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{ExtractionCursor, SourceConnector, TargetConnector};

    #[test]
    fn erp_connector_id_and_display() {
        let c = ErpConnector;
        assert_eq!(c.source_id(), SourceId("erp.sovereign"));
        assert_eq!(c.display_name(), "ERP System");
    }

    #[test]
    fn crm_schema_has_required_email() {
        let c = CrmConnector;
        let hashes = c.schema().required_hashes();
        assert!(hashes.contains(&0x2003));
    }

    #[test]
    fn excel_extract_returns_empty_batch() {
        let c = ExcelFileConnector;
        let batch = c.extract(&ExtractionCursor::default()).unwrap();
        assert!(batch.is_empty());
    }

    #[test]
    fn dw_target_accepts_batch() {
        let t = DataWarehouseTarget;
        let batch = DataBatch::new(SourceId("test"), vec![vec![(1u32, b"v".to_vec())]], 1);
        let receipt = t.deliver(batch).unwrap();
        assert_eq!(receipt.records_accepted, 1);
    }

    #[test]
    fn notification_target_id_correct() {
        let t = NotificationTarget;
        assert_eq!(t.target_id(), TargetId("notify.sovereign"));
    }

    #[test]
    fn all_source_adapters_have_unique_ids() {
        let ids = vec![
            ErpConnector.source_id(),
            CrmConnector.source_id(),
            HrSystemConnector.source_id(),
            LegacySystemConnector.source_id(),
            ExternalPartnerConnector.source_id(),
            ExcelFileConnector.source_id(),
            EmailInboxConnector.source_id(),
            ThirdPartyApiConnector.source_id(),
        ];
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
    }

    #[test]
    fn all_target_adapters_have_unique_ids() {
        let ids = vec![
            DataWarehouseTarget.target_id(),
            ReportingToolsTarget.target_id(),
            DashboardTarget.target_id(),
            OtherApplicationsTarget.target_id(),
            FileExportTarget.target_id(),
            ExternalPortalTarget.target_id(),
            NotificationTarget.target_id(),
        ];
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
    }
}
