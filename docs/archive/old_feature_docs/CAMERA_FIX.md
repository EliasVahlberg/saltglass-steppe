# Camera Jitter Fix

## Issue
Camera centering calculations caused jitter with odd-numbered viewport dimensions due to fractional division results.

## Solution
- Changed from floating-point division (`view_width as f32 / 2.0`) to integer division (`view_width / 2`)
- This ensures consistent pixel-perfect centering without fractional offsets
- Eliminates camera jitter when viewport has odd dimensions

## Testing
- Added DES test scenario `camera_centering_test.json` 
- Verified player movement doesn't cause camera positioning issues
- All existing DES scenarios continue to pass

## Files Modified
- `src/renderer/camera.rs`: Fixed centering calculation
- `tests/scenarios/camera_centering_test.json`: Added test scenario
