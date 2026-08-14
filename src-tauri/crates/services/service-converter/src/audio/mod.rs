pub mod av3a;
pub mod kgg;
pub mod kgm;
pub mod kwm;
pub mod ncm;
pub mod qmc;

pub use av3a::{extract_av3a_stream, find_audio_track, inspect_mp4_audio_codec};
pub use kgg::{candidate_kugou_db_paths, convert_kgg_bytes_with_ekey, decrypt_pc_database, derive_page_aes_iv, derive_page_aes_key, next_page_iv};
pub use kgm::{convert_kgma_bytes, kugo_md5, xor_collapse_u32};
pub use kwm::{convert_kwm_bytes, generate_kwm_mask};
pub use ncm::{convert_ncm_bytes, detect_audio_format, NcmDecryptedResult, NcmMetadata};
pub use qmc::{convert_qmc_bytes, create_qmc2_cipher, decrypt_tencent_tea, Qmc2Map, Qmc2Rc4};
