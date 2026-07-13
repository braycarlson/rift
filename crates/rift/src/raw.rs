use std::borrow::Cow;

use bytes::Bytes;

use crate::RiotApi;
use crate::client::{Auth, RequestPlan, path_encode};
use crate::error::Error;
use crate::generated::models::match_v5::MatchDto;
use crate::generated::routes::RegionalRoute;

const MATCH_V5_GET_MATCH_ID: &str = "match-v5.getMatch";
const MATCH_V5_GET_MATCH_PATH: &str = "/lol/match/v5/matches";

impl RiotApi {
    /// Get a match, returning both the parsed [`MatchDto`] and the exact
    /// response body Riot sent.
    ///
    /// The typed endpoint methods deserialize into DTOs, so re-serializing a
    /// DTO silently drops any field the models do not capture. Archival
    /// callers should persist the returned bytes instead; the full payload
    /// then survives model changes. Shares the `match-v5.getMatch` rate limit
    /// bucket, retries, and cache with the typed variant.
    ///
    /// # Errors
    ///
    /// Returns the same errors as `match_v5_get_match`; `Ok(None)` on 404.
    pub async fn match_v5_get_match_raw(
        &self,
        route: RegionalRoute,
        match_id: &str,
    ) -> Result<Option<(MatchDto, Bytes)>, Error> {
        assert!(!match_id.is_empty(), "match id must not be empty");

        let path = Cow::Owned(format!(
            "{MATCH_V5_GET_MATCH_PATH}/{}",
            path_encode(match_id)
        ));
        let query_pairs: Vec<(&'static str, String)> = Vec::new();

        let plan = RequestPlan {
            auth: Auth::ApiKey,
            body: None,
            endpoint_id: MATCH_V5_GET_MATCH_ID,
            method: "GET",
            path,
            query: query_pairs,
            route: route.as_str(),
        };

        self.execute_optional_raw(plan).await
    }
}

#[cfg(test)]
mod tests {
    use crate::generated::meta::ENDPOINTS;

    use super::{MATCH_V5_GET_MATCH_ID, MATCH_V5_GET_MATCH_PATH};

    #[test]
    fn raw_variant_matches_generated_metadata() {
        let endpoint = ENDPOINTS
            .iter()
            .find(|endpoint| endpoint.id == MATCH_V5_GET_MATCH_ID)
            .expect("match-v5.getMatch present in generated metadata");

        assert_eq!(endpoint.method, "GET");
        assert_eq!(endpoint.path, "/lol/match/v5/matches/{matchId}");
        assert!(endpoint.path.starts_with(MATCH_V5_GET_MATCH_PATH));
    }
}
