use crate::RiotApi;
use crate::error::Error;
use crate::generated::endpoints::MatchV5GetMatchIdsByPuuidQuery;
use crate::generated::models::league_v4::LeagueEntryDto;
use crate::generated::routes::{PlatformRoute, RegionalRoute};

#[doc(hidden)]
pub const LEAGUE_PAGES_MAX: u32 = 1_000;
#[doc(hidden)]
pub const MATCH_IDS_ALL_MAX: u32 = 10_000;
const MATCH_IDS_PAGE_MAX: i32 = 100;
const PAGES_MARGIN: u32 = 1;

impl RiotApi {
    /// Walks `match-v5` match ids across pages until `ids_max` are collected or
    /// the history is exhausted.
    ///
    /// The `count` and `start` fields of `query` are managed internally; any
    /// other filters (queue, type, time bounds) are preserved on every page.
    ///
    /// # Errors
    ///
    /// Returns the first error from any underlying page request.
    pub async fn match_v5_match_ids_all(
        &self,
        route: RegionalRoute,
        puuid: &str,
        query: &MatchV5GetMatchIdsByPuuidQuery<'_>,
        ids_max: u32,
    ) -> Result<Vec<String>, Error> {
        assert!(!puuid.is_empty(), "puuid must not be empty");
        assert!(
            ids_max <= MATCH_IDS_ALL_MAX,
            "ids_max exceeds {MATCH_IDS_ALL_MAX}"
        );

        let mut collected: Vec<String> = Vec::with_capacity(ids_max as usize);
        let mut start: i32 = query.start.unwrap_or(0);
        let pages_max = ids_max.div_ceil(MATCH_IDS_PAGE_MAX.unsigned_abs()) + PAGES_MARGIN;
        let mut pages: u32 = 0;

        loop {
            pages += 1;

            assert!(pages <= pages_max, "pagination exceeded {pages_max} pages");

            let remaining = (ids_max as usize).saturating_sub(collected.len());

            if remaining == 0 {
                break;
            }

            let count = i32::try_from(remaining)
                .unwrap_or(MATCH_IDS_PAGE_MAX)
                .min(MATCH_IDS_PAGE_MAX);

            let page_query = MatchV5GetMatchIdsByPuuidQuery {
                count: Some(count),
                start: Some(start),
                ..*query
            };

            let page = self
                .match_v5_get_match_ids_by_puuid(route, puuid, &page_query)
                .await?;

            let page_len = page.len();

            collected.extend(page);

            if page_len < count.unsigned_abs() as usize {
                break;
            }

            start += count;
        }

        assert!(
            collected.len() <= ids_max as usize,
            "collected exceeds ids_max"
        );

        Ok(collected)
    }

    /// Walks every `league-v4` entry page for a queue, tier, and division.
    ///
    /// Stops at the first empty page, a 404 (absent division), or `pages_max`.
    ///
    /// # Errors
    ///
    /// Returns the first error from any underlying page request.
    pub async fn league_v4_entries_all(
        &self,
        route: PlatformRoute,
        queue: &str,
        tier: &str,
        division: &str,
        pages_max: u32,
    ) -> Result<Vec<LeagueEntryDto>, Error> {
        assert!(!queue.is_empty(), "queue must not be empty");
        assert!(!tier.is_empty(), "tier must not be empty");
        assert!(!division.is_empty(), "division must not be empty");
        assert!(
            pages_max <= LEAGUE_PAGES_MAX,
            "pages_max exceeds {LEAGUE_PAGES_MAX}"
        );

        let mut collected: Vec<LeagueEntryDto> = Vec::new();
        let mut page: i32 = 1;
        let mut iterations: u32 = 0;

        loop {
            iterations += 1;

            if iterations > pages_max {
                break;
            }

            let entries = self
                .league_v4_get_league_entries(route, queue, tier, division, Some(page))
                .await?;

            let Some(entries) = entries else {
                break;
            };

            if entries.is_empty() {
                break;
            }

            collected.extend(entries);
            page += 1;
        }

        assert!(
            iterations <= pages_max + 1,
            "walk must not exceed the page cap"
        );

        Ok(collected)
    }
}
