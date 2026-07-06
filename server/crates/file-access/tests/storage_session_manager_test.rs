use file_access::storage_session::NbdIndexPool;

#[test]
fn nbd_index_pool_releases_index_for_reuse() {
    let mut pool = NbdIndexPool::new(1);

    let first = pool.acquire().unwrap();
    assert_eq!(first, 0);
    assert!(pool.acquire().is_none());

    pool.release(first);
    assert_eq!(pool.acquire(), Some(0));
}

#[test]
fn nbd_index_pool_release_is_idempotent() {
    let mut pool = NbdIndexPool::new(1);

    pool.release(0);
    pool.release(0);

    assert_eq!(pool.acquire(), Some(0));
    assert!(pool.acquire().is_none());
}
