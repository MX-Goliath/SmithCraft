#[cfg(windows)]
pub(super) fn choose_color(current: [u8; 3], custom: &mut [u32; 16]) -> Option<[u8; 3]> {
    use windows_sys::Win32::UI::Controls::Dialogs::{CHOOSECOLORW, ChooseColorW};
    // Use the same Windows common color dialog as the reference application.
    unsafe {
        let mut dialog: CHOOSECOLORW = std::mem::zeroed();
        dialog.lStructSize = std::mem::size_of::<CHOOSECOLORW>() as u32;
        dialog.rgbResult =
            current[0] as u32 | ((current[1] as u32) << 8) | ((current[2] as u32) << 16);
        dialog.lpCustColors = custom.as_mut_ptr();
        if ChooseColorW(&mut dialog) != 0 {
            Some([
                dialog.rgbResult as u8,
                (dialog.rgbResult >> 8) as u8,
                (dialog.rgbResult >> 16) as u8,
            ])
        } else {
            None
        }
    }
}
#[cfg(not(windows))]
pub(super) fn choose_color(_current: [u8; 3], _custom: &mut [u32; 16]) -> Option<[u8; 3]> {
    None
}
