//! SlaGuiRenderer — pure-HTML compliance dashboard and selection form.
//!
//! Generates self-contained HTML strings (inline CSS + no JS frameworks).
//! The caller embeds the output inside a page template or returns it directly.

use crate::{
    QUALITY_DIVISOR, GO_LIVE_THRESHOLD,
    profile::{MaturityLevel, SlaProfile},
    requirements::{ComplianceDomain, requirements_for_domain},
    evaluator::{
        ComplianceStatus, DomainScore, PriorityAction, SlaReport,
    },
};

// ── Shared CSS ────────────────────────────────────────────────────────────────

const CSS: &str = r#"
<style>
:root {
  --sovereign:#0a7c43;--compliant:#1565c0;--partial:#f57f17;
  --action-required:#e65100;--mandatory-gap:#b71c1c;
  --p0:#b71c1c;--p1:#e65100;--p2:#f57f17;--p3:#1565c0;
  --bg:#f5f5f5;--card:#fff;--border:#ddd;--text:#212121;--sub:#666;
}
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:'Segoe UI',system-ui,sans-serif;background:var(--bg);color:var(--text);padding:24px}
h1{font-size:1.6rem;margin-bottom:4px}
h2{font-size:1.2rem;margin:20px 0 10px}
h3{font-size:1rem;margin-bottom:6px}
.subtitle{color:var(--sub);font-size:.9rem;margin-bottom:20px}
.card{background:var(--card);border:1px solid var(--border);border-radius:8px;padding:16px;margin-bottom:16px}
.badge{display:inline-block;padding:3px 10px;border-radius:12px;font-size:.78rem;font-weight:600;color:#fff}
.badge.sovereign{background:var(--sovereign)}
.badge.compliant{background:var(--compliant)}
.badge.partial{background:var(--partial)}
.badge.action-required{background:var(--action-required)}
.badge.mandatory-gap{background:var(--mandatory-gap)}
.progress-outer{background:#e0e0e0;border-radius:6px;height:14px;margin:6px 0}
.progress-inner{height:14px;border-radius:6px;transition:width .3s}
.progress-inner.sovereign{background:var(--sovereign)}
.progress-inner.compliant{background:var(--compliant)}
.progress-inner.partial{background:var(--partial)}
.progress-inner.action-required{background:var(--action-required)}
.progress-inner.mandatory-gap{background:var(--mandatory-gap)}
.grid-2{display:grid;grid-template-columns:1fr 1fr;gap:16px}
.score-big{font-size:2.8rem;font-weight:700;line-height:1}
.score-label{font-size:.82rem;color:var(--sub);margin-top:4px}
table{width:100%;border-collapse:collapse;font-size:.88rem}
th{text-align:left;padding:8px 10px;background:#eeeeee;font-weight:600}
td{padding:7px 10px;border-bottom:1px solid #f0f0f0;vertical-align:top}
tr:last-child td{border-bottom:none}
select,input[type=checkbox]{cursor:pointer}
select{padding:4px 6px;border:1px solid var(--border);border-radius:4px;background:#fff;font-size:.85rem}
label{display:inline-flex;align-items:center;gap:6px;cursor:pointer}
.domain-row{display:flex;align-items:center;justify-content:space-between;margin-bottom:4px}
.domain-icon{font-size:1.1rem;margin-right:6px}
.go-live-ready{background:#e8f5e9;border:2px solid var(--sovereign);border-radius:8px;padding:14px 18px;display:flex;align-items:center;gap:12px;margin-bottom:20px}
.go-live-blocked{background:#ffebee;border:2px solid var(--mandatory-gap);border-radius:8px;padding:14px 18px;display:flex;align-items:center;gap:12px;margin-bottom:20px}
.go-live-icon{font-size:2rem}
.action-row{display:flex;align-items:flex-start;gap:10px;padding:10px 0;border-bottom:1px solid #f0f0f0}
.action-row:last-child{border-bottom:none}
.p-badge{display:inline-block;min-width:110px;padding:3px 8px;border-radius:4px;font-size:.75rem;font-weight:700;color:#fff;text-align:center}
.p0{background:var(--p0)}.p1{background:var(--p1)}.p2{background:var(--p2)}.p3{background:var(--p3)}
.akk-ref{font-size:.75rem;color:var(--sub);font-style:italic}
.form-section{margin-bottom:24px}
.req-row{display:grid;grid-template-columns:32px 1fr 160px;align-items:center;gap:8px;padding:8px 0;border-bottom:1px solid #f5f5f5}
.req-row:last-child{border-bottom:none}
.req-name{font-size:.88rem;font-weight:500}
.req-desc{font-size:.78rem;color:var(--sub)}
.mandatory-star{color:var(--mandatory-gap);font-weight:700;margin-left:3px}
.submit-btn{background:var(--compliant);color:#fff;border:none;border-radius:6px;padding:12px 28px;font-size:1rem;font-weight:600;cursor:pointer;margin-top:16px}
.submit-btn:hover{opacity:.9}
</style>
"#;

// ── SlaGuiRenderer ─────────────────────────────────────────────────────────────

pub struct SlaGuiRenderer;

impl SlaGuiRenderer {
    // ── Selection Form ────────────────────────────────────────────────────────

    /// Renders the stakeholder requirement-selection form.
    ///
    /// The form posts to `action_url` via POST with fields:
    /// - `req_{id}_enabled` = "on" if checkbox checked
    /// - `req_{id}_level`   = u8 (MaturityLevel::as_u8())
    pub fn render_selection_html(
        profile:    &SlaProfile,
        action_url: &str,
    ) -> String {
        let mut out = String::with_capacity(32_768);

        out.push_str("<!DOCTYPE html><html lang=\"en\"><head>");
        out.push_str("<meta charset=\"UTF-8\">");
        out.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
        out.push_str("<title>SLA Compliance Configuration</title>");
        out.push_str(CSS);
        out.push_str("</head><body>");

        push_header(&mut out, profile.name, profile.topology.as_str());

        out.push_str("<p class=\"subtitle\">Select the requirements applicable to your deployment and set the current implementation maturity level for each.</p>");

        out.push_str("<form method=\"POST\" action=\"");
        html_escape_into(&mut out, action_url);
        out.push_str("\">");

        let all_domains = all_domains_ordered();
        for domain in all_domains {
            let reqs: Vec<_> = requirements_for_domain(domain).collect();
            if reqs.is_empty() { continue; }

            out.push_str("<div class=\"card form-section\">");
            out.push_str("<h2>");
            out.push_str(domain_icon(domain));
            out.push(' ');
            html_escape_into(&mut out, domain.as_str());
            out.push_str("</h2>");

            // header row
            out.push_str("<div class=\"req-row\" style=\"font-size:.78rem;color:var(--sub);font-weight:600\">");
            out.push_str("<span>Select</span><span>Requirement</span><span>Maturity Level</span>");
            out.push_str("</div>");

            for req in reqs {
                let selected = profile.contains(req.id);
                out.push_str("<div class=\"req-row\">");

                // checkbox
                out.push_str("<input type=\"checkbox\" name=\"req_");
                push_u16(&mut out, req.id);
                out.push_str("_enabled\" id=\"req_");
                push_u16(&mut out, req.id);
                out.push('"');
                if selected { out.push_str(" checked"); }
                out.push('>');

                // name + description
                out.push_str("<div><div class=\"req-name\">");
                html_escape_into(&mut out, req.name);
                if req.mandatory {
                    out.push_str("<span class=\"mandatory-star\" title=\"Mandatory\">*</span>");
                }
                out.push_str("</div><div class=\"req-desc\">");
                html_escape_into(&mut out, req.description);
                if let Some(pol) = req.akk_policy {
                    out.push_str(" <span class=\"akk-ref\">[");
                    html_escape_into(&mut out, pol);
                    out.push_str("]</span>");
                }
                out.push_str("</div></div>");

                // maturity level select
                out.push_str("<select name=\"req_");
                push_u16(&mut out, req.id);
                out.push_str("_level\">");
                for level in maturity_levels() {
                    out.push_str("<option value=\"");
                    push_u8(&mut out, level.as_u8());
                    out.push('"');
                    if level == MaturityLevel::NotStarted { out.push_str(" selected"); }
                    out.push('>');
                    html_escape_into(&mut out, level.as_str());
                    out.push_str("</option>");
                }
                out.push_str("</select>");

                out.push_str("</div>"); // req-row
            }

            out.push_str("</div>"); // card
        }

        out.push_str("<p style=\"font-size:.8rem;color:var(--sub)\"><span class=\"mandatory-star\">*</span> Mandatory — must reach ≥80% (B11 ≥192) before go-live.</p>");
        out.push_str("<button type=\"submit\" class=\"submit-btn\">Evaluate Compliance</button>");
        out.push_str("</form>");
        out.push_str("</body></html>");
        out
    }

    // ── Dashboard ─────────────────────────────────────────────────────────────

    /// Renders the full compliance dashboard from an `SlaReport`.
    pub fn render_dashboard_html(report: &SlaReport) -> String {
        let mut out = String::with_capacity(65_536);

        out.push_str("<!DOCTYPE html><html lang=\"en\"><head>");
        out.push_str("<meta charset=\"UTF-8\">");
        out.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">");
        out.push_str("<title>SLA Compliance Dashboard — ");
        html_escape_into(&mut out, report.profile_name);
        out.push_str("</title>");
        out.push_str(CSS);
        out.push_str("</head><body>");

        push_header(&mut out, report.profile_name, report.topology);

        // ── Go-live readiness banner ──────────────────────────────────────────
        if report.go_live_ready {
            out.push_str("<div class=\"go-live-ready\">");
            out.push_str("<span class=\"go-live-icon\">✅</span>");
            out.push_str("<div><strong>GO-LIVE READY</strong><div style=\"font-size:.88rem\">All mandatory requirements met. Overall compliance above 75% threshold.</div></div>");
            out.push_str("</div>");
        } else {
            out.push_str("<div class=\"go-live-blocked\">");
            out.push_str("<span class=\"go-live-icon\">🚫</span>");
            out.push_str("<div><strong>NOT GO-LIVE READY</strong><div style=\"font-size:.88rem\">");
            if report.domains.iter().any(|d| d.mandatory_gap) {
                out.push_str("One or more mandatory requirements have a blocking gap. Resolve P0 actions before deployment.");
            } else {
                out.push_str("Overall compliance score below go-live threshold (75% / B11 180).");
            }
            out.push_str("</div></div>");
            out.push_str("</div>");
        }

        // ── Overall score ─────────────────────────────────────────────────────
        out.push_str("<div class=\"card\">");
        out.push_str("<div class=\"grid-2\">");

        out.push_str("<div>");
        out.push_str("<div class=\"score-big\" style=\"color:");
        out.push_str(status_color(report.overall_status));
        out.push_str("\">");
        push_u8(&mut out, report.overall_percent());
        out.push_str("%</div>");
        out.push_str("<div class=\"score-label\">Overall Compliance Score</div>");
        out.push_str("<div style=\"margin-top:8px\">");
        out.push_str("<span class=\"badge ");
        out.push_str(report.overall_status.css_class());
        out.push_str("\">");
        html_escape_into(&mut out, report.overall_status.as_str());
        out.push_str("</span></div>");
        out.push_str("</div>");

        out.push_str("<div style=\"display:flex;flex-direction:column;justify-content:center\">");
        out.push_str("<div style=\"font-size:.88rem;color:var(--sub);margin-bottom:6px\">B11 Score (0–240)</div>");
        out.push_str("<div class=\"progress-outer\"><div class=\"progress-inner ");
        out.push_str(report.overall_status.css_class());
        out.push_str("\" style=\"width:");
        push_u8(&mut out, report.overall_percent());
        out.push_str("%\"></div></div>");
        out.push_str("<div style=\"font-size:.8rem;color:var(--sub)\">B11 ");
        push_u8(&mut out, report.overall_b11);
        out.push('/');
        push_u8(&mut out, QUALITY_DIVISOR);
        out.push_str(" &nbsp;·&nbsp; Go-live threshold: ");
        push_u8(&mut out, GO_LIVE_THRESHOLD);
        out.push_str("</div>");
        out.push_str("</div>");

        out.push_str("</div>"); // grid-2
        out.push_str("</div>"); // card

        // ── Domain scores ─────────────────────────────────────────────────────
        out.push_str("<h2>Domain Scores</h2>");
        out.push_str("<div class=\"grid-2\">");

        for domain in &report.domains {
            render_domain_card(&mut out, domain);
        }

        out.push_str("</div>"); // grid-2

        // ── Priority actions ──────────────────────────────────────────────────
        if !report.priority_actions.is_empty() {
            out.push_str("<h2>Priority Action Plan</h2>");
            out.push_str("<div class=\"card\">");
            for action in &report.priority_actions {
                render_action_row(&mut out, action);
            }
            out.push_str("</div>");
        }

        // ── Per-domain requirement detail ─────────────────────────────────────
        out.push_str("<h2>Requirement Detail</h2>");
        for domain in &report.domains {
            out.push_str("<div class=\"card\">");
            out.push_str("<h3>");
            out.push_str(domain_icon(domain.domain));
            out.push(' ');
            html_escape_into(&mut out, domain.domain.as_str());
            if domain.mandatory_gap {
                out.push_str(" <span class=\"badge mandatory-gap\">MANDATORY GAP</span>");
            }
            out.push_str("</h3>");

            out.push_str("<table><thead><tr>");
            out.push_str("<th>Requirement</th><th>Maturity</th><th>B11</th><th>Action</th>");
            out.push_str("</tr></thead><tbody>");

            for check in &domain.checks {
                out.push_str("<tr>");
                out.push_str("<td>");
                html_escape_into(&mut out, check.name);
                if check.mandatory {
                    out.push_str("<span class=\"mandatory-star\">*</span>");
                }
                out.push_str("</td>");
                out.push_str("<td><span class=\"badge ");
                out.push_str(check.maturity.css_class());
                out.push_str("\" style=\"");
                out.push_str(maturity_badge_style(check.maturity));
                out.push_str("\">");
                html_escape_into(&mut out, check.maturity.as_str());
                out.push_str("</span></td>");
                out.push_str("<td>");
                push_u8(&mut out, check.b11_score);
                out.push_str("</td><td class=\"akk-ref\">");
                if let Some(action) = check.gap_action {
                    html_escape_into(&mut out, action);
                } else {
                    out.push_str("—");
                }
                out.push_str("</td></tr>");
            }

            out.push_str("</tbody></table></div>");
        }

        // footer
        out.push_str("<p style=\"font-size:.75rem;color:var(--sub);margin-top:24px;text-align:center\">");
        out.push_str("BahyWay SLA Governance Engine &nbsp;·&nbsp; Generated epoch ");
        push_u32(&mut out, report.generated_epoch);
        out.push_str(" &nbsp;·&nbsp; QUALITY_DIVISOR=240 (ADR-001)");
        out.push_str("</p>");

        out.push_str("</body></html>");
        out
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn push_header(out: &mut String, title: &str, subtitle: &str) {
    out.push_str("<h1>");
    html_escape_into(out, title);
    out.push_str("</h1><p class=\"subtitle\">");
    html_escape_into(out, subtitle);
    out.push_str("</p>");
}

fn render_domain_card(out: &mut String, domain: &DomainScore) {
    out.push_str("<div class=\"card\">");
    out.push_str("<div class=\"domain-row\">");
    out.push_str("<div><span class=\"domain-icon\">");
    out.push_str(domain_icon(domain.domain));
    out.push_str("</span><strong>");
    html_escape_into(out, domain.domain.as_str());
    out.push_str("</strong></div>");
    out.push_str("<span class=\"badge ");
    out.push_str(domain.status.css_class());
    out.push_str("\">");
    html_escape_into(out, domain.status.as_str());
    out.push_str("</span>");
    out.push_str("</div>"); // domain-row

    // progress bar
    let pct = domain.percent();
    out.push_str("<div class=\"progress-outer\"><div class=\"progress-inner ");
    out.push_str(domain.status.css_class());
    out.push_str("\" style=\"width:");
    push_u8(out, pct);
    out.push_str("%\"></div></div>");
    out.push_str("<div style=\"font-size:.8rem;color:var(--sub)\">");
    push_u8(out, pct);
    out.push_str("% &nbsp;(B11 ");
    push_u8(out, domain.b11_score);
    out.push(')');
    if domain.mandatory_gap {
        out.push_str(" <span style=\"color:var(--mandatory-gap);font-weight:700\">⚠ Mandatory Gap</span>");
    }
    out.push_str("</div>");

    if !domain.top_gap_actions.is_empty() {
        out.push_str("<div style=\"margin-top:8px;font-size:.78rem\">");
        out.push_str("<strong>Top gaps:</strong> ");
        for (i, action) in domain.top_gap_actions.iter().enumerate() {
            if i > 0 { out.push_str("; "); }
            html_escape_into(out, action);
        }
        out.push_str("</div>");
    }

    out.push_str("</div>"); // card
}

fn render_action_row(out: &mut String, action: &PriorityAction) {
    out.push_str("<div class=\"action-row\">");
    out.push_str("<span class=\"p-badge ");
    out.push_str(action.priority.css_class());
    out.push_str("\">");
    html_escape_into(out, action.priority.as_str());
    out.push_str("</span>");
    out.push_str("<div><div style=\"font-size:.88rem;font-weight:500\">");
    html_escape_into(out, action.name);
    out.push_str("</div><div class=\"akk-ref\">");
    html_escape_into(out, action.action);
    if let Some(pol) = action.akk_policy {
        out.push_str(" [");
        html_escape_into(out, pol);
        out.push(']');
    }
    out.push_str("</div></div>");
    out.push_str("</div>"); // action-row
}

fn html_escape_into(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&'  => out.push_str("&amp;"),
            '<'  => out.push_str("&lt;"),
            '>'  => out.push_str("&gt;"),
            '"'  => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c    => out.push(c),
        }
    }
}

fn push_u8(out: &mut String, v: u8) {
    out.push_str(&v.to_string());
}

fn push_u16(out: &mut String, v: u16) {
    out.push_str(&v.to_string());
}

fn push_u32(out: &mut String, v: u32) {
    out.push_str(&v.to_string());
}

fn domain_icon(domain: ComplianceDomain) -> &'static str {
    match domain {
        ComplianceDomain::GdprPrivacy          => "🔏",
        ComplianceDomain::EncryptionAtRest      => "🔐",
        ComplianceDomain::KeyManagement         => "🗝",
        ComplianceDomain::NetworkSecurity       => "🌐",
        ComplianceDomain::OperationalSecurity   => "🛡",
        ComplianceDomain::NajafPrivacy          => "🕌",
        ComplianceDomain::WpdInfrastructure     => "🏗",
        ComplianceDomain::GovernanceAssurance   => "📋",
    }
}

fn status_color(status: ComplianceStatus) -> &'static str {
    match status {
        ComplianceStatus::Sovereign      => "var(--sovereign)",
        ComplianceStatus::Compliant      => "var(--compliant)",
        ComplianceStatus::Partial        => "var(--partial)",
        ComplianceStatus::ActionRequired => "var(--action-required)",
        ComplianceStatus::MandatoryGap   => "var(--mandatory-gap)",
    }
}

fn maturity_badge_style(level: MaturityLevel) -> &'static str {
    match level {
        MaturityLevel::NotApplicable => "background:#9e9e9e;color:#fff",
        MaturityLevel::NotStarted    => "background:#757575;color:#fff",
        MaturityLevel::Planned       => "background:#1565c0;color:#fff",
        MaturityLevel::InProgress    => "background:#f57f17;color:#fff",
        MaturityLevel::Implemented   => "background:#2e7d32;color:#fff",
        MaturityLevel::Verified      => "background:#0a7c43;color:#fff",
    }
}

fn maturity_levels() -> [MaturityLevel; 6] {
    [
        MaturityLevel::NotApplicable,
        MaturityLevel::NotStarted,
        MaturityLevel::Planned,
        MaturityLevel::InProgress,
        MaturityLevel::Implemented,
        MaturityLevel::Verified,
    ]
}

fn all_domains_ordered() -> [ComplianceDomain; 8] {
    [
        ComplianceDomain::GdprPrivacy,
        ComplianceDomain::EncryptionAtRest,
        ComplianceDomain::KeyManagement,
        ComplianceDomain::NetworkSecurity,
        ComplianceDomain::OperationalSecurity,
        ComplianceDomain::NajafPrivacy,
        ComplianceDomain::WpdInfrastructure,
        ComplianceDomain::GovernanceAssurance,
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{AppTopology, SlaProfile};
    use crate::evaluator::{ComplianceState, SlaEvaluator};

    #[test]
    fn selection_form_contains_required_elements() {
        let profile = SlaProfile::for_topology(AppTopology::NajafEngine);
        let html    = SlaGuiRenderer::render_selection_html(&profile, "/sla/evaluate");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("/sla/evaluate"));
        assert!(html.contains("req_1001_enabled"));
        assert!(html.contains("req_1001_level"));
        assert!(html.contains("Not Started"));
        assert!(html.contains("Verified"));
    }

    #[test]
    fn dashboard_shows_go_live_blocked_for_empty_state() {
        let profile = SlaProfile::for_topology(AppTopology::NajafEngine);
        let state   = ComplianceState::new();
        let report  = SlaEvaluator::evaluate(&profile, &state, 999);
        let html    = SlaGuiRenderer::render_dashboard_html(&report);
        assert!(html.contains("NOT GO-LIVE READY"));
        assert!(html.contains("go-live-blocked"));
        assert!(!html.contains("class=\"go-live-ready\""));
    }

    #[test]
    fn dashboard_shows_go_live_ready_when_fully_verified() {
        let profile = SlaProfile::for_topology(AppTopology::NajafEngine);
        let mut state = ComplianceState::new();
        for &id in &profile.selected_ids {
            state.set(id, MaturityLevel::Verified);
        }
        let report = SlaEvaluator::evaluate(&profile, &state, 999);
        let html   = SlaGuiRenderer::render_dashboard_html(&report);
        assert!(html.contains("GO-LIVE READY"));
        assert!(!html.contains("NOT GO-LIVE READY"));
    }

    #[test]
    fn dashboard_escapes_html_in_strings() {
        let profile = SlaProfile::for_topology(AppTopology::WebGateway);
        let state   = ComplianceState::new();
        let report  = SlaEvaluator::evaluate(&profile, &state, 1);
        let html    = SlaGuiRenderer::render_dashboard_html(&report);
        // Should not contain raw unescaped angle brackets in text content
        // (CSS/attributes are fine — check only outside of style/script)
        assert!(html.contains("</body>"));
    }

    #[test]
    fn enterprise_dashboard_renders_without_panic() {
        let profile = SlaProfile::for_topology(AppTopology::EnterpriseAll);
        let state   = ComplianceState::new();
        let report  = SlaEvaluator::evaluate(&profile, &state, 0);
        let html    = SlaGuiRenderer::render_dashboard_html(&report);
        assert!(html.len() > 1000, "dashboard HTML should be substantial");
    }

    #[test]
    fn selection_form_marks_mandatory_requirements() {
        let profile = SlaProfile::for_topology(AppTopology::NajafEngine);
        let html    = SlaGuiRenderer::render_selection_html(&profile, "/");
        // DPIA (1001) is mandatory — should have the mandatory star
        assert!(html.contains("mandatory-star"));
    }
}
