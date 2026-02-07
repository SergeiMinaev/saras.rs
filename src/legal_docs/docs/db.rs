use crate::legal_docs::docs::models::Doc;
use crate::legal_docs::docs::forms::DocForm;
use lpsql::pool::ConnectionPool;
use lpsql::Lpsql;
use std::sync::Arc;

pub struct DocDb {
    pool: Arc<ConnectionPool>,
}

impl DocDb {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        DocDb { pool }
    }

    pub async fn page(&self, offset: i32, size: i32) -> Vec<Doc> {
        let q = "select row_to_json(data) from (
            select id, key, title as label, title, html, version, is_active
            from legal_docs_docs
            order by id offset $1::INT limit $2::INT
        ) data";
        let items = Lpsql::query(q).bind(offset).bind(size).fetch_all(&self.pool).await;
        items.into_iter().map(|json| serde_json::from_str(&json).unwrap()).collect()
    }

    pub async fn by_id(&self, id: i32) -> Option<Doc> {
        let query = "select row_to_json(data) from (
            select id, key, title as label, title, html, version, is_active
            from legal_docs_docs where id = $1::INT
        ) data";
        Lpsql::query(query).bind(id).fetch_one(&self.pool).await
            .and_then(|v| serde_json::from_str(&v).ok())
    }

    pub async fn by_key(&self, key: &str) -> Option<Doc> {
        let query = "select row_to_json(data) from (
            select id, key, title as label, title, html, version, is_active
            from legal_docs_docs where key = $1::TEXT
            order by id desc limit 1
        ) data";
        Lpsql::query(query).bind(key).fetch_one(&self.pool).await
            .and_then(|v| serde_json::from_str(&v).ok())
    }

    pub async fn active_by_key(&self, key: &str) -> Option<Doc> {
        let query = "select row_to_json(data) from (
            select id, key, title as label, title, html, version, is_active
            from legal_docs_docs where key = $1::TEXT and is_active = true
            order by id desc limit 1
        ) data";
        Lpsql::query(query).bind(key).fetch_one(&self.pool).await
            .and_then(|v| serde_json::from_str(&v).ok())
    }

    pub async fn total_count(&self) -> i32 {
        let q = "select count(*) from legal_docs_docs";
        Lpsql::query(q).fetch_one(&self.pool).await.unwrap().parse().unwrap()
    }

    pub async fn create(&self, data: DocForm) -> Option<i32> {
        let q = "insert into legal_docs_docs (key, title, html, version, is_active)
            values ($1::TEXT, $2::TEXT, $3::TEXT, $4::TEXT, $5::BOOL) returning id";
        Lpsql::query(q)
            .bind(data.key)
            .bind(data.title)
            .bind(data.html)
            .bind(data.version)
            .bind(data.is_active)
            .fetch_one(&self.pool).await
            .map(|id| id.parse().unwrap())
    }

    pub async fn create_and_get(&self, form: DocForm) -> Option<Doc> {
        let id = self.create(form).await?;
        self.by_id(id).await
    }

    pub async fn update(&self, id: i32, data: DocForm) -> Option<i32> {
        let q = "update legal_docs_docs set key = $2::TEXT,
            title = $3::TEXT,
            html = $4::TEXT,
            version = $5::TEXT,
            is_active = $6::BOOL
            where id = $1::INT
            returning id";
        Lpsql::query(q)
            .bind(id)
            .bind(data.key)
            .bind(data.title)
            .bind(data.html)
            .bind(data.version)
            .bind(data.is_active)
            .fetch_one(&self.pool).await
            .map(|id| id.parse().unwrap())
    }

    pub async fn update_and_get(&self, id: i32, data: DocForm) -> Option<Doc> {
        let id = self.update(id, data).await?;
        self.by_id(id).await
    }

    pub async fn delete(&self, id: i32) -> bool {
        let q = "delete from legal_docs_docs where id = $1::INT";
        Lpsql::query(q).bind(id).exec(&self.pool).await != 0
    }
}
