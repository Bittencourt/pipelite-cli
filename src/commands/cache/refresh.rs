use anyhow::{bail, Result};

use crate::api::{
    ActivitiesListParams, DealsListParams, OrgsListParams, PeopleListParams, PipelinesListParams,
    StagesListParams,
};
use crate::cache::{
    KEY_ACTIVITIES, KEY_DEALS, KEY_ORGS, KEY_PEOPLE, KEY_PIPELINES, KEY_STAGES, TTL_ENTITY_LIST,
    TTL_PIPELINES, TTL_STAGES,
};
use crate::context::AppContext;

/// Fetch all cacheable entities from the API and store them locally.
///
/// Uses auto-paginate (batch of 100, loop until fewer than limit returned).
/// Stores as Vec<(String, String)> tuples (id, display_name) for each entity type.
pub async fn run(ctx: &AppContext) -> Result<()> {
    let store = match ctx.cache {
        Some(ref s) => s,
        None => bail!("Cache is not available (could not create cache directory)"),
    };

    let mut total_entities: usize = 0;

    // ── Pipelines ──────────────────────────────────────────────────
    {
        let items = fetch_all_pipelines(ctx).await?;
        let count = items.len();
        store.set(KEY_PIPELINES, &items, TTL_PIPELINES)?;
        total_entities += count;

        // For each pipeline, cache its stages
        for (pipeline_id, _name) in &items {
            let stages = fetch_all_stages(ctx, pipeline_id).await?;
            let stage_count = stages.len();
            let key = format!("{}_{}", KEY_STAGES, pipeline_id);
            store.set(&key, &stages, TTL_STAGES)?;
            total_entities += stage_count;
        }
    }

    // ── Deals ──────────────────────────────────────────────────────
    {
        let items = fetch_all_deals(ctx).await?;
        let count = items.len();
        store.set(KEY_DEALS, &items, TTL_ENTITY_LIST)?;
        total_entities += count;
    }

    // ── Organizations ──────────────────────────────────────────────
    {
        let items = fetch_all_orgs(ctx).await?;
        let count = items.len();
        store.set(KEY_ORGS, &items, TTL_ENTITY_LIST)?;
        total_entities += count;
    }

    // ── People ─────────────────────────────────────────────────────
    {
        let items = fetch_all_people(ctx).await?;
        let count = items.len();
        store.set(KEY_PEOPLE, &items, TTL_ENTITY_LIST)?;
        total_entities += count;
    }

    // ── Activities ─────────────────────────────────────────────────
    {
        let items = fetch_all_activities(ctx).await?;
        let count = items.len();
        store.set(KEY_ACTIVITIES, &items, TTL_ENTITY_LIST)?;
        total_entities += count;
    }

    if !ctx.quiet {
        eprintln!("Cache refreshed. {} entities cached.", total_entities);
    }

    Ok(())
}

/// Auto-paginate: fetch all pipelines as (id, name) tuples.
async fn fetch_all_pipelines(ctx: &AppContext) -> Result<Vec<(String, String)>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 100u64;

    loop {
        let params = PipelinesListParams {
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_pipelines(&params).await?;
        let count = response.data.len();
        for item in response.data {
            all.push((item.id, item.name));
        }
        if (count as u64) < limit {
            break;
        }
        offset += limit;
    }

    Ok(all)
}

/// Auto-paginate: fetch all stages for a given pipeline as (id, name) tuples.
async fn fetch_all_stages(ctx: &AppContext, pipeline_id: &str) -> Result<Vec<(String, String)>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 100u64;

    loop {
        let params = StagesListParams {
            pipeline_id: pipeline_id.to_string(),
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_stages(&params).await?;
        let count = response.data.len();
        for item in response.data {
            all.push((item.id, item.name));
        }
        if (count as u64) < limit {
            break;
        }
        offset += limit;
    }

    Ok(all)
}

/// Auto-paginate: fetch all deals as (id, title) tuples.
async fn fetch_all_deals(ctx: &AppContext) -> Result<Vec<(String, String)>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 100u64;

    loop {
        let params = DealsListParams {
            stage: None,
            org: None,
            owner: None,
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_deals(&params).await?;
        let count = response.data.len();
        for item in response.data {
            all.push((item.id, item.title));
        }
        if (count as u64) < limit {
            break;
        }
        offset += limit;
    }

    Ok(all)
}

/// Auto-paginate: fetch all organizations as (id, name) tuples.
async fn fetch_all_orgs(ctx: &AppContext) -> Result<Vec<(String, String)>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 100u64;

    loop {
        let params = OrgsListParams {
            owner: None,
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_orgs(&params).await?;
        let count = response.data.len();
        for item in response.data {
            all.push((item.id, item.name));
        }
        if (count as u64) < limit {
            break;
        }
        offset += limit;
    }

    Ok(all)
}

/// Auto-paginate: fetch all people as (id, display_name) tuples.
async fn fetch_all_people(ctx: &AppContext) -> Result<Vec<(String, String)>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 100u64;

    loop {
        let params = PeopleListParams {
            org: None,
            owner: None,
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_people(&params).await?;
        let count = response.data.len();
        for item in response.data {
            let name = format!("{} {}", item.first_name, item.last_name)
                .trim()
                .to_string();
            all.push((item.id, name));
        }
        if (count as u64) < limit {
            break;
        }
        offset += limit;
    }

    Ok(all)
}

/// Auto-paginate: fetch all activities as (id, title) tuples.
async fn fetch_all_activities(ctx: &AppContext) -> Result<Vec<(String, String)>> {
    let mut all = Vec::new();
    let mut offset = 0u64;
    let limit = 100u64;

    loop {
        let params = ActivitiesListParams {
            type_id: None,
            deal_id: None,
            owner_id: None,
            limit,
            offset,
            expand: None,
        };
        let response = ctx.client.list_activities(&params).await?;
        let count = response.data.len();
        for item in response.data {
            all.push((item.id, item.title));
        }
        if (count as u64) < limit {
            break;
        }
        offset += limit;
    }

    Ok(all)
}
