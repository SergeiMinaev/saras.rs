use crate::http::{Pagination, Request};

// Общие хелперы для admin-списков, управляемых схемой (контракт с front_admin):
// постраничная выборка (page/size) и сортировка (sort_by/sort_dir).

pub struct PageParams {
    pub page: u64,
    pub per_page: u64,
    pub offset: i32,
    pub size: i32,
}

pub fn page_params(req: &Request) -> PageParams {
    let page = req
        .query
        .get("page")
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(1);
    let per_page = req
        .query
        .get("size")
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(1, 200))
        .unwrap_or(20);
    PageParams {
        page,
        per_page,
        offset: ((page - 1) * per_page) as i32,
        size: per_page as i32,
    }
}

pub fn make_pagination(page: u64, per_page: u64, total: i64) -> Pagination {
    let total_u64 = if total < 0 { 0 } else { total as u64 };
    let total_pages = if total_u64 == 0 {
        Some(1)
    } else {
        Some(((total_u64 + per_page - 1) / per_page).max(1))
    };
    Pagination {
        page,
        per_page,
        total: Some(total_u64),
        total_pages,
        next_page: total_pages.and_then(|tp| if page < tp { Some(page + 1) } else { None }),
        prev_page: if page > 1 { Some(page - 1) } else { None },
    }
}

pub mod sort {
    use crate::http::Request;

    // Имя колонки нельзя передать bind-параметром, поэтому sort_by валидируется
    // по allowlist реальных колонок модели; иначе — дефолтный порядок модели.

    pub fn from_req(req: &Request) -> (Option<String>, Option<String>) {
        let by = req.query.get("sort_by").map(|s| s.to_string());
        let dir = req.query.get("sort_dir").map(|s| s.to_string());
        (by, dir)
    }

    pub fn order_by_clause(
        sort_by: Option<&str>,
        sort_dir: Option<&str>,
        allowed: &[&str],
        default: &str,
    ) -> String {
        match sort_by.filter(|f| allowed.contains(f)) {
            Some(field) => {
                let dir = if sort_dir == Some("asc") { "asc" } else { "desc" };
                format!("order by {field} {dir}")
            }
            None => format!("order by {default}"),
        }
    }
}
