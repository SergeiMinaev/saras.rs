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
    parse_page_params(
        req.query.get("page").map(|s| s.as_str()),
        req.query.get("size").map(|s| s.as_str()),
    )
}

fn parse_page_params(page: Option<&str>, size: Option<&str>) -> PageParams {
    let page = page
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(1);
    let per_page = size
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::sort::order_by_clause;

    const ALLOWED: &[&str] = &["id", "name"];
    const DEFAULT: &str = "id desc";

    #[test]
    fn order_by_valid_field_both_dirs() {
        assert_eq!(order_by_clause(Some("name"), Some("asc"), ALLOWED, DEFAULT), "order by name asc");
        assert_eq!(order_by_clause(Some("name"), Some("desc"), ALLOWED, DEFAULT), "order by name desc");
    }

    #[test]
    fn order_by_dir_defaults_to_desc() {
        // Любое значение кроме "asc" (None, мусор) трактуется как desc.
        assert_eq!(order_by_clause(Some("id"), None, ALLOWED, DEFAULT), "order by id desc");
        assert_eq!(order_by_clause(Some("id"), Some("ASC"), ALLOWED, DEFAULT), "order by id desc");
        assert_eq!(order_by_clause(Some("id"), Some("; drop"), ALLOWED, DEFAULT), "order by id desc");
    }

    #[test]
    fn order_by_rejects_field_outside_allowlist() {
        // Защита от инъекции: невалидное поле -> дефолтный порядок, чужой текст в SQL не попадает.
        assert_eq!(
            order_by_clause(Some("name; drop table users"), Some("asc"), ALLOWED, DEFAULT),
            "order by id desc"
        );
        assert_eq!(order_by_clause(Some("created_at"), Some("asc"), ALLOWED, DEFAULT), "order by id desc");
    }

    #[test]
    fn order_by_no_field_uses_default() {
        assert_eq!(order_by_clause(None, Some("asc"), ALLOWED, DEFAULT), "order by id desc");
    }

    #[test]
    fn page_params_defaults() {
        let p = parse_page_params(None, None);
        assert_eq!((p.page, p.per_page, p.offset, p.size), (1, 20, 0, 20));
    }

    #[test]
    fn page_params_parses_and_computes_offset() {
        let p = parse_page_params(Some("3"), Some("50"));
        assert_eq!((p.page, p.per_page, p.offset, p.size), (3, 50, 100, 50));
    }

    #[test]
    fn page_params_clamps_and_guards() {
        // size клампится в 1..200, page < 1 и мусор -> дефолты.
        assert_eq!(parse_page_params(Some("2"), Some("999")).per_page, 200);
        assert_eq!(parse_page_params(Some("2"), Some("0")).per_page, 1);
        assert_eq!(parse_page_params(Some("0"), None).page, 1);
        let p = parse_page_params(Some("abc"), Some("xyz"));
        assert_eq!((p.page, p.per_page), (1, 20));
    }

    #[test]
    fn pagination_empty() {
        let pg = make_pagination(1, 20, 0);
        assert_eq!(pg.total, Some(0));
        assert_eq!(pg.total_pages, Some(1));
        assert_eq!(pg.next_page, None);
        assert_eq!(pg.prev_page, None);
    }

    #[test]
    fn pagination_middle_and_last_page() {
        let mid = make_pagination(2, 20, 45);
        assert_eq!(mid.total_pages, Some(3));
        assert_eq!(mid.next_page, Some(3));
        assert_eq!(mid.prev_page, Some(1));

        let last = make_pagination(3, 20, 45);
        assert_eq!(last.next_page, None);
        assert_eq!(last.prev_page, Some(2));
    }
}
