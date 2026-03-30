use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DashboardStats {
    pub total_properties: i64,
    pub free_properties: i64,
    pub total_clients: i64,
    pub active_deals: i64,
    pub monthly_revenue: f64,
}

#[derive(Debug, Clone, FromRow)]
pub struct RecentDealRow {
    pub id: Uuid,
    pub property_title: String,
    pub client_name: String,
    pub manager_name: String,
    pub deal_type_name: String,
    pub deal_status_name: String,
    pub amount: f64,
    pub start_date: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct CatalogReportRow {
    pub property_title: String,
    pub category_name: String,
    pub status_name: String,
    pub district_name: String,
    pub owner_name: String,
    pub price: f64,
}

#[derive(Debug, Clone, FromRow)]
pub struct OwnerPortfolioRow {
    pub owner_name: String,
    pub properties_count: i64,
    pub active_deals: i64,
    pub total_portfolio_value: f64,
}

#[derive(Debug, Clone, FromRow)]
pub struct MonthlySalesRow {
    pub period_label: String,
    pub completed_deals: i64,
    pub total_amount: f64,
    pub total_commission: f64,
    pub payments_received: f64,
}

#[derive(Debug, Clone, FromRow)]
pub struct ManagerPerformanceRow {
    pub manager_name: String,
    pub completed_deals: i64,
    pub total_amount: f64,
    pub total_commission: f64,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuditLogRow {
    pub actor_name: String,
    pub action_type: String,
    pub entity_name: String,
    pub details: String,
    pub created_at: String,
}
