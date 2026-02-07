use crate::legal_docs::docs::db::DocDb;
use crate::legal_docs::docs::forms::DocForm;
use crate::legal_docs::docs::models::Doc;
use crate::errors::Error;
use crate::db::get_pool;

pub async fn create_doc(form: DocForm) -> Result<Doc, Error> {
    let pool = get_pool();
    let docdb = DocDb::new(pool.clone());
    docdb.create_and_get(form).await.ok_or(Error::Common)
}

pub async fn update_doc(id: i32, form: DocForm) -> Result<Doc, Error> {
    let pool = get_pool();
    let docdb = DocDb::new(pool.clone());
    docdb.update_and_get(id, form).await.ok_or(Error::Common)
}

pub async fn delete_doc(id: i32) -> bool {
    let pool = get_pool();
    let docdb = DocDb::new(pool.clone());
    docdb.delete(id).await
}
