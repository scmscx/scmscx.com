#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LangData {
    English,
    Korean,
}

#[derive(Clone, Debug)]
pub struct TrackingAnalytics {
    pub tracking_analytics_id: String,
    pub was_provided_by_request: bool,
}
