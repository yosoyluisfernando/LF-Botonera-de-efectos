//! Identidad opaca de MMDevice; el nombre visible puede cambiar sin perderla.
use windows::{
    core::PWSTR,
    Win32::{
        Devices::Properties::DEVPKEY_Device_FriendlyName,
        Media::Audio::{eRender, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE},
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
            StructuredStorage::PropVariantToStringAlloc, CLSCTX_ALL, COINIT_MULTITHREADED,
            STGM_READ,
        },
        UI::Shell::PropertiesSystem::PROPERTYKEY,
    },
};
struct Apartment(bool);
impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
unsafe fn owned_string(raw: PWSTR) -> String {
    let value = raw.to_string().unwrap_or_default();
    CoTaskMemFree(Some(raw.0.cast()));
    value
}
pub fn identities() -> Vec<(String, String)> {
    unsafe { enumerate().unwrap_or_default() }
}
unsafe fn enumerate() -> windows::core::Result<Vec<(String, String)>> {
    // Un apartamento ya inicializado con otro modelo puede seguir usando MMDevice.
    let result = CoInitializeEx(None, COINIT_MULTITHREADED);
    let _apartment = Apartment(result.is_ok());
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
    let outputs = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
    let mut found = Vec::new();
    let key = PROPERTYKEY {
        fmtid: DEVPKEY_Device_FriendlyName.fmtid,
        pid: DEVPKEY_Device_FriendlyName.pid,
    };
    for i in 0..outputs.GetCount()? {
        let read = || -> windows::core::Result<(String, String)> {
            let device = outputs.Item(i)?;
            let id = owned_string(device.GetId()?);
            let properties = device.OpenPropertyStore(STGM_READ)?;
            let name = properties.GetValue(&key)?;
            let name = owned_string(PropVariantToStringAlloc(&name)?);
            Ok((id, name))
        };
        if let Ok((id, name)) = read() {
            if !id.is_empty() && !name.is_empty() {
                found.push((format!("wasapi:{id}"), name));
            }
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Requiere salidas de audio de Windows; solo enumera, no abre streams"]
    fn native_ids_are_available_on_this_machine() {
        let entries = super::identities();
        assert!(!entries.is_empty());
        assert!(entries
            .iter()
            .all(|(id, name)| id.starts_with("wasapi:") && !name.is_empty()));
        let catalog = super::super::device_catalog::refresh();
        assert!(catalog.iter().any(|d| !d.id.is_empty() && d.usable));
    }
}
