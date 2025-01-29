use crate::core::business::BusinessHandle;
use crate::core::event::system::fetch_rkey::FetchRKeyEvent;
use std::sync::Arc;

impl BusinessHandle {
    pub async fn test_01(self: &Arc<Self>) -> crate::Result<()> {
        tracing::info!("Test 01 start");
        tracing::info!("Test 01 end");
        Ok(())
    }

    pub async fn fetch_rkey(self: &Arc<Self>) -> crate::Result<()> {
        let mut fetch_event = FetchRKeyEvent {};
        let res = self.send_event(&mut fetch_event).await?;
        tracing::info!("FetchRKeyEvent: {:?}", res);
        Ok(())
    }
}
