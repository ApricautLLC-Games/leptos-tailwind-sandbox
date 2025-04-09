#[cfg(test)]
mod data_export_tests {
    use leptos::*;
    use leptos::prelude::*;
    use wasm_bindgen_test::*;
    use wasm_bindgen::JsValue;
    use web_sys::Storage;
    use crate::utils::localStorage;
    use crate::test_utils::{click_and_wait,get_by_test_id};
    use crate::theme::ThemeProvider;
    use crate::data::{DataButton,export_data};
    use gloo_timers::future::TimeoutFuture;
    use serde_json::{Value, json};
    
    wasm_bindgen_test_configure!(run_in_browser);

    // Helper to wait for async operations to complete
    async fn wait_for_storage() {
        TimeoutFuture::new(50).await;
    }
    
    // Helper to reset localStorage for tests
    async fn reset_storage() {
        localStorage::reset_all_storage();
        wait_for_storage().await;
    }

    #[wasm_bindgen_test]
    async fn test_load_button_exists() {
        // Reset storage to ensure a clean state
        reset_storage().await;
        
        // Mount the DataButton component
        mount_to_body(|| view! {
            <ThemeProvider>
                <DataButton />
            </ThemeProvider>
        });
        
        // Click the data button to show the panel
        let data_button = get_by_test_id("data-button");
        click_and_wait(&data_button, 50).await;
        
        // Verify the data panel is shown
        let data_panel = get_by_test_id("data-panel");
        
        // Check for the load button
        let load_button = get_by_test_id("load-data-button");
    }
    
    #[wasm_bindgen_test]
    async fn test_export_button_exists() {
        // Reset storage to ensure a clean state
        reset_storage().await;
        
        // Mount the DataButton component
        mount_to_body(|| view! {
            <ThemeProvider>
                <DataButton />
            </ThemeProvider>
        });
        
        // Click the data button to show the panel
        let data_button = get_by_test_id("data-button");
        click_and_wait(&data_button, 50).await;
        
        // Verify the data panel is shown
        let data_panel = get_by_test_id("data-panel");
        
        // Check for the export button
        let export_button = get_by_test_id("export-data-button");
    }
    
    #[wasm_bindgen_test]
    async fn test_export_button_triggers_download() {
        // This test is more challenging to fully automate due to browser download behavior
        // We'll check that clicking the button calls our export function
        // and prepares a download link with the correct content type
        
        // Reset storage for clean test
        reset_storage().await;
        
        // Mount the DataButton component
        mount_to_body(|| view! {
            <ThemeProvider>
                <DataButton />
            </ThemeProvider>
        });
        
        // Open the data panel
        let data_button = get_by_test_id("data-button");
        click_and_wait(&data_button, 50).await;
        
        // Click the export button
        let export_button = get_by_test_id("export-data-button");
        click_and_wait(&export_button, 50).await;
        
        // Since we can't directly test the download, we'll check for the confirmation message
        let export_success = get_by_test_id("export-success-message");
        
        // Verify the success message contains expected text
        let message_text = export_success.inner_html();
        assert!(message_text.contains("Data exported"), "Success message should indicate data was exported");
    }
    
    #[wasm_bindgen_test]
    async fn test_export_data_structure() {
        // Call the export function to get the JSON data
        let json_data = export_data().expect("Export should succeed in tests");
        
        // Verify that it returns some data
        assert!(!json_data.is_empty(), "Export should return non-empty JSON string");
        
        // Try to parse it as valid JSON
        let parsed: Result<Value, _> = serde_json::from_str(&json_data);
        assert!(parsed.is_ok(), "Export should return valid JSON data");
        
        let data = parsed.unwrap();
        
        // Check structure: should be an object with specific fields
        assert!(data.is_object(), "Exported data should be a JSON object");
        
        // Check for required fields
        assert!(data.get("version").is_some(), "Exported data should include a version field");
        assert!(data.get("timestamp").is_some(), "Exported data should include a timestamp field");
        assert!(data.get("data").is_some(), "Exported data should include a data field");
        
        // Check data field structure
        let data_field = data.get("data").unwrap();
        assert!(data_field.is_object(), "Data field should be a JSON object");
        
        // Verify expected data keys exist
        let data_obj = data_field.as_object().unwrap();
        assert!(data_obj.contains_key("player_id"), "Data should include player_id");
        assert!(data_obj.contains_key("dark_mode"), "Data should include dark_mode");
    }
    
    #[wasm_bindgen_test]
    async fn test_export_data_error_handling() {
        // Test to ensure errors are properly returned from export_data
        // We'll simulate an error by temporarily removing player_id from storage
        
        // First store the current player_id value to restore later
        let player_id_backup = localStorage::get_storage_item("player_id").unwrap();
        
        // Remove player_id from storage
        let _ = crate::utils::remove_storage_item("player_id");
        wait_for_storage().await;
        
        // Attempt to export data
        let result = export_data();
        
        // Verify we get an error
        assert!(result.is_err(), "Export should return an error when player_id is missing");
        
        // Check that the error message mentions player ID
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("player ID"), "Error should mention missing player ID");
        
        // Restore player_id if it existed
        if let Some(id) = player_id_backup {
            localStorage::set_storage_item("player_id", &id);
            wait_for_storage().await;
        }
    }
    
    #[wasm_bindgen_test]
    async fn test_exported_data_matches_storage() {
        // Reset storage with known values
        reset_storage().await;
        
        // Create a known player ID for testing
        let test_player_id = "test_player_123";
        localStorage::set_storage_item("player_id", test_player_id);
        
        // Set a known dark mode value
        localStorage::set_storage_item("dark_mode", "true");
        
        // Wait for storage operations to complete
        wait_for_storage().await;
        
        // Get exported data
        let json_data = export_data().expect("Export should succeed with valid test data");
        
        // Parse the data
        let parsed: Value = serde_json::from_str(&json_data).unwrap();
        let data_field = parsed.get("data").unwrap().as_object().unwrap();
        
        // Compare with actual storage values
        let exported_player_id = data_field.get("player_id").unwrap().as_str().unwrap();
        let exported_dark_mode = data_field.get("dark_mode").unwrap().as_bool().unwrap();
        
        // Verify player_id matches
        assert_eq!(exported_player_id, test_player_id, "Exported player_id should match storage");
        
        // Verify dark_mode matches
        assert!(exported_dark_mode, "Exported dark_mode should match 'true' from storage");
    }
    
    #[wasm_bindgen_test]
    async fn test_exported_data_can_be_parsed_for_import() {
        // Reset storage with valid data
        reset_storage().await;
        localStorage::set_storage_item("player_id", "test_import_id");
        localStorage::set_storage_item("dark_mode", "false");
        wait_for_storage().await;
        
        // Export data
        let json_data = export_data().expect("Export should succeed with valid test data");
        
        // This test just verifies that the data could be used for import
        // by parsing it and checking that the fields needed for import are accessible
        let parsed: Value = serde_json::from_str(&json_data).unwrap();
        
        // Access all fields that would be needed for import
        let version = parsed.get("version").unwrap().as_str().unwrap();
        let data = parsed.get("data").unwrap().as_object().unwrap();
        
        // Check version is a semantic version string (basic check)
        assert!(version.contains('.'), "Version should be in semantic version format");
        
        // Verify we can get the player_id as a string
        let player_id = data.get("player_id").unwrap().as_str().unwrap();
        assert_eq!(player_id, "test_import_id", "player_id should match the test value");
        
        // Verify we can get dark_mode as a boolean
        let dark_mode = data.get("dark_mode").unwrap().as_bool().unwrap();
        assert_eq!(dark_mode, false, "dark_mode should match the test value");
    }
}