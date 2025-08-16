// Container for all tag related types and methods.
//
// Note that while this module is named "tag_internal", a lot of these types are exposed publicly,
// and even the whole hls module is public. The "internal" naming is based on the fact that the
// module is not directly exposed as public, and furthermore, there *is* a public module that we
// call "tag", and so we needed some sort of disambiguation.
pub mod hls;
pub mod known;
pub mod unknown;
pub mod value;
