   Compiling enclave-runner v0.1.0 (/home/parth/code/rust-sgx/enclave-runner)
warning: unused import: `futures::prelude::*`
  --> enclave-runner/src/command.rs:15:5
   |
15 | use futures::prelude::*;
   |     ^^^^^^^^^^^^^^^^^^^
   |
   = note: #[warn(unused_imports)] on by default

warning: unused import: `std::thread`
  --> enclave-runner/src/usercalls/mod.rs:22:5
   |
22 | use std::thread;
   |     ^^^^^^^^^^^

warning: unused import: `tokio::prelude::*`
  --> enclave-runner/src/usercalls/mod.rs:34:5
   |
34 | use tokio::prelude::*;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `futures::future::lazy`
  --> enclave-runner/src/usercalls/mod.rs:35:5
   |
35 | use futures::future::lazy;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `futures::future::Future`
  --> enclave-runner/src/usercalls/mod.rs:37:5
   |
37 | use futures::future::Future;
   |     ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::usercalls::abi::Register`
  --> enclave-runner/src/usercalls/mod.rs:52:5
   |
52 | use crate::usercalls::abi::Register;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `crate::usercalls::abi::DispatchResult`
  --> enclave-runner/src/usercalls/mod.rs:53:5
   |
53 | use crate::usercalls::abi::DispatchResult;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

#![feature(prelude_import)]
#![no_std]
/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(asm)]
#![feature(proc_macro, generators, custom_attribute)]
#![feature(async_await, await_macro, futures_api)]
#![feature(proc_macro_hygiene)]
#![doc(html_logo_url = "https://edp.fortanix.com/img/docs/edp-logo.svg",
       html_favicon_url = "https://edp.fortanix.com/favicon.ico",
       html_root_url = "https://edp.fortanix.com/docs/api/")]
#[prelude_import]
use ::std::prelude::v1::*;
#[macro_use]
extern crate std as std;

extern crate openssl;
extern crate sgx_isa;
extern crate sgxs;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate fnv;
extern crate fortanix_sgx_abi;
#[macro_use]
extern crate lazy_static;
extern crate futures;
#[macro_use]
extern crate tokio;

mod command {

    use std::path::Path;
    use failure::Error;
    use sgxs::loader::{Load, MappingInfo};
    use crate::loader::{EnclaveBuilder, ErasedTcs};
    use std::os::raw::c_void;
    use crate::usercalls::EnclaveState;
    use futures::prelude::*;
    pub struct Command {
        main: ErasedTcs,
        threads: Vec<ErasedTcs>,
        address: *mut c_void,
        size: usize,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::fmt::Debug for Command {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                Command {
                main: ref __self_0_0,
                threads: ref __self_0_1,
                address: ref __self_0_2,
                size: ref __self_0_3 } => {
                    let mut debug_trait_builder = f.debug_struct("Command");
                    let _ =
                        debug_trait_builder.field("main", &&(*__self_0_0));
                    let _ =
                        debug_trait_builder.field("threads", &&(*__self_0_1));
                    let _ =
                        debug_trait_builder.field("address", &&(*__self_0_2));
                    let _ =
                        debug_trait_builder.field("size", &&(*__self_0_3));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl MappingInfo for Command {
        fn address(&self) -> *mut c_void { self.address }
        fn size(&self) -> usize { self.size }
    }
    impl Command {
        /// # Panics
        /// Panics if the number of TCSs is 0.
        pub(crate) fn internal_new(mut tcss: Vec<ErasedTcs>,
                                   address: *mut c_void, size: usize)
         -> Command {
            let main = tcss.remove(0);
            Command{main, threads: tcss, address, size,}
        }
        pub fn new<P: AsRef<Path>, L: Load>(enclave_path: P, loader: &mut L)
         -> Result<Command, Error> {
            EnclaveBuilder::new(enclave_path.as_ref()).build(loader)
        }
        pub fn run(self) -> Result<(), Error> {
            futures::executor::block_on(EnclaveState::main_entry(self.main,
                                                                 self.threads))
        }
    }
}
mod library {
    use std::path::Path;
    use std::sync::Arc;
    use failure::Error;
    use sgxs::loader::{Load, MappingInfo};
    use crate::loader::{EnclaveBuilder, ErasedTcs};
    use std::fmt;
    use std::os::raw::c_void;
    use crate::usercalls::EnclaveState;
    pub struct Library {
        enclave: Arc<EnclaveState>,
        address: *mut c_void,
        size: usize,
    }
    impl fmt::Debug for Library {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Library").field("address",
                                            &self.address).field("size",
                                                                 &self.size).finish()
        }
    }
    impl MappingInfo for Library {
        fn address(&self) -> *mut c_void { self.address }
        fn size(&self) -> usize { self.size }
    }
    impl Library {
        pub(crate) fn internal_new(tcss: Vec<ErasedTcs>, address: *mut c_void,
                                   size: usize) -> Library {
            Library{enclave: EnclaveState::library(tcss), address, size,}
        }
        pub fn new<P: AsRef<Path>, L: Load>(enclave_path: P, loader: &mut L)
         -> Result<Library, Error> {
            EnclaveBuilder::new(enclave_path.as_ref()).build_library(loader)
        }
        /// If this library's TCSs are all currently servicing other calls, this
        /// function will block until a TCS becomes available.
        ///
        /// # Safety
        ///
        /// The caller must ensure that the parameters passed-in match what the
        /// enclave is expecting.
        pub unsafe fn call(&self, p1: u64, p2: u64, p3: u64, p4: u64, p5: u64)
         -> Result<(u64, u64), Error> {
            let enclave_clone = self.enclave.clone();
            futures::executor::block_on(EnclaveState::library_entry(enclave_clone,
                                                                    p1, p2,
                                                                    p3, p4,
                                                                    p5))
        }
    }
}
mod loader {
    use std::fs::File;
    use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult};
    use std::os::raw::c_void;
    use std::path::Path;
    use std::{arch, str};
    use failure::{Error, ResultExt};
    use openssl::hash::Hasher;
    use openssl::pkey::PKey;
    use sgx_isa::{Attributes, AttributesFlags, Miscselect, Sigstruct};
    use sgxs::loader::{Load, MappingInfo, Tcs};
    use sgxs::sigstruct::{self, EnclaveHash, Signer};
    use crate::tcs::DebugBuffer;
    use crate::{Command, Library};
    enum EnclaveSource<'a> { Path(&'a Path), File(File), Data(&'a [u8]), }
    impl <'a> EnclaveSource<'a> {
        fn try_clone(&self) -> Option<Self> {
            match *self {
                EnclaveSource::Path(path) => Some(EnclaveSource::Path(path)),
                EnclaveSource::Data(data) => Some(EnclaveSource::Data(data)),
                EnclaveSource::File(_) => None,
            }
        }
    }
    impl <'a> Read for EnclaveSource<'a> {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            if let &mut EnclaveSource::Path(path) = self {
                let file = File::open(path)?;
                *self = EnclaveSource::File(file);
            }
            match *self {
                EnclaveSource::File(ref mut file) => file.read(buf),
                EnclaveSource::Data(ref mut data) => data.read(buf),
                EnclaveSource::Path(_) => {
                    {
                        ::std::rt::begin_panic("internal error: entered unreachable code",
                                               &("enclave-runner/src/loader.rs",
                                                 51u32, 39u32))
                    }
                }
            }
        }
    }
    pub struct EnclaveBuilder<'a> {
        enclave: EnclaveSource<'a>,
        signature: Option<Sigstruct>,
        attributes: Option<Attributes>,
        miscselect: Option<Miscselect>,
    }
    pub enum EnclavePanic {

        /// The first byte of the debug buffer was 0
        #[fail(display = "Enclave panicked.")]
        NoDebugBuf,

        /// The debug buffer could be interpreted as a zero-terminated UTF-8 string
        #[fail(display = "Enclave panicked: {}", _0)]
        DebugStr(String),

        /// The first byte of the debug buffer was not 0, but it was also not a
        /// zero-terminated UTF-8 string
        #[fail(display = "Enclave panicked: {:?}", _0)]
        DebugBuf(Vec<u8>),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::fmt::Debug for EnclavePanic {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match (&*self,) {
                (&EnclavePanic::NoDebugBuf,) => {
                    let mut debug_trait_builder = f.debug_tuple("NoDebugBuf");
                    debug_trait_builder.finish()
                }
                (&EnclavePanic::DebugStr(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("DebugStr");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&EnclavePanic::DebugBuf(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("DebugBuf");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::failure::Fail for EnclavePanic {
        #[allow(unreachable_code)]
        fn cause(&self) -> ::std::option::Option<&::failure::Fail> {
            match *self {
                EnclavePanic::NoDebugBuf => { return None }
                EnclavePanic::DebugStr(ref __binding_0) => { return None }
                EnclavePanic::DebugBuf(ref __binding_0) => { return None }
            }
            None
        }
        #[allow(unreachable_code)]
        fn backtrace(&self) -> ::std::option::Option<&::failure::Backtrace> {
            match *self {
                EnclavePanic::NoDebugBuf => { return None }
                EnclavePanic::DebugStr(ref __binding_0) => { return None }
                EnclavePanic::DebugBuf(ref __binding_0) => { return None }
            }
            None
        }
    }
    impl ::std::fmt::Display for EnclavePanic {
        #[allow(unreachable_code)]
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                EnclavePanic::NoDebugBuf => {
                    return f.write_fmt(::std::fmt::Arguments::new_v1(&["Enclave panicked."],
                                                                     &match ()
                                                                          {
                                                                          ()
                                                                          =>
                                                                          [],
                                                                      }))
                }
                EnclavePanic::DebugStr(ref __binding_0) => {
                    return f.write_fmt(::std::fmt::Arguments::new_v1(&["Enclave panicked: "],
                                                                     &match (&__binding_0,)
                                                                          {
                                                                          (arg0,)
                                                                          =>
                                                                          [::std::fmt::ArgumentV1::new(arg0,
                                                                                                       ::std::fmt::Display::fmt)],
                                                                      }))
                }
                EnclavePanic::DebugBuf(ref __binding_0) => {
                    return f.write_fmt(::std::fmt::Arguments::new_v1(&["Enclave panicked: "],
                                                                     &match (&__binding_0,)
                                                                          {
                                                                          (arg0,)
                                                                          =>
                                                                          [::std::fmt::ArgumentV1::new(arg0,
                                                                                                       ::std::fmt::Debug::fmt)],
                                                                      }))
                }
            }
            f.write_fmt(::std::fmt::Arguments::new_v1(&["An error has occurred."],
                                                      &match () {
                                                           () => [],
                                                       }))
        }
    }
    impl From<DebugBuffer> for EnclavePanic {
        fn from(buf: DebugBuffer) -> EnclavePanic {
            if buf[0] == 0 {
                EnclavePanic::NoDebugBuf
            } else {
                match str::from_utf8(buf.split(|v| *v == 0).next().unwrap()) {
                    Ok(s) => EnclavePanic::DebugStr(s.to_owned()),
                    Err(_) => EnclavePanic::DebugBuf(buf.to_vec()),
                }
            }
        }
    }
    pub(crate) struct ErasedTcs {
        address: *mut c_void,
        tcs: Box<Tcs>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::fmt::Debug for ErasedTcs {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                ErasedTcs { address: ref __self_0_0, tcs: ref __self_0_1 } =>
                {
                    let mut debug_trait_builder = f.debug_struct("ErasedTcs");
                    let _ =
                        debug_trait_builder.field("address", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("tcs", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    unsafe impl Send for ErasedTcs { }
    impl ErasedTcs {
        fn new<T: Tcs + 'static>(tcs: T) -> ErasedTcs {
            ErasedTcs{address: tcs.address(), tcs: Box::new(tcs),}
        }
    }
    impl Tcs for ErasedTcs {
        fn address(&self) -> *mut c_void { self.address }
    }
    impl <'a> EnclaveBuilder<'a> {
        pub fn new(enclave_path: &'a Path) -> EnclaveBuilder<'a> {
            let mut ret =
                EnclaveBuilder{enclave: EnclaveSource::Path(enclave_path),
                               attributes: None,
                               miscselect: None,
                               signature: None,};
            let _ = ret.coresident_signature();
            ret
        }
        pub fn new_from_memory(enclave_data: &'a [u8]) -> EnclaveBuilder<'a> {
            let mut ret =
                EnclaveBuilder{enclave: EnclaveSource::Data(enclave_data),
                               attributes: None,
                               miscselect: None,
                               signature: None,};
            let _ = ret.coresident_signature();
            ret
        }
        fn generate_dummy_signature(&self) -> Result<Sigstruct, Error> {
            fn xgetbv0() -> u64 { unsafe { arch::x86_64::_xgetbv(0) } }
            let mut enclave = self.enclave.try_clone().unwrap();
            let mut signer =
                Signer::new(EnclaveHash::from_stream::<_,
                                                       Hasher>(&mut enclave)?);
            let attributes =
                self.attributes.unwrap_or_else(||
                                                   Attributes{flags:
                                                                  AttributesFlags::DEBUG
                                                                      |
                                                                      AttributesFlags::MODE64BIT,
                                                              xfrm:
                                                                  xgetbv0(),});
            signer.attributes_flags(attributes.flags,
                                    !0).attributes_xfrm(attributes.xfrm, !0);
            if let Some(miscselect) = self.miscselect {
                signer.miscselect(miscselect, !0);
            }
            let key =
                PKey::private_key_from_der(b"0\x82\x06\xe2\x02\x01\x00\x02\x82\x01\x81\x00\xb1\xb0\x17\xe2\xcf\xbb\x90r)\x1f\xe6U\x04\xab]y\xf0\x9f1\xa7)\x81\x02\xfb\xda\x889\xaf\x88J\xa2=%\xe6\xdc\x83\xa4}\xf2\xa7\xe4\xe9\xd7?\xbb\x11\n\x89\x0b\x9b\xdbB\xe5\xf4S\xa5\x82\xa0\xdcl\xe7\xf9\xfd,\x9c\xbfo\xbet)\xe7\xba4J\x0c\xec_bH9\x1a\xf7j/M%\xbb5\x8a\xe41JQ\x08\xb1\xd1 \xc5J)\x1d\xf4\xb0\'\xf0=ue\xaec\x1eT\xf8\xd1{\xfe\x84\xfaV\xd6}<\xedl\xbe\x87\x1a\xbe\xbd\xc8\xdb\xed\xaa\xa1HM-\xf5{\xd7\x87\x90\x8c\xb4r\xca\xe2\x08h\x1aFjm\x91\xeb\xb2Ls+\xb9\xab\\s\x0b\xb5!O\x96XO\xef\xfe\x7f\x0c\x84\xeea,\r\xb3&\x98\xda\xed\xaa\x165\xdc\xcb\xf0{\xf4^\xf4/\x83\xdf\xd5\x1c\xff\xe4m\xd4\xa9_[7\x06\x1a\xa1\xc3\x94\xf6\x88\xf3\xb5%k\xf0\xf1\x0b\x9ci\x05\xf0$C\xf2\x82\xd0\xaaX\xed,a~\x9b\x8f\x8e\x12\xd4\xbb@\x10\x17\xa4\x04E\xb3j\x1e\xa3\xb0\xca\xd6\xbbZ\x03\r\xfb\x84\x14\xb3rs\xb5BM<\xe9\xbc\xf2V\x94oJ7|x\xa8$\x8f@@\x1d\xb8I}\xcf\x00yZ\xa7\x9dj\x81\xee\x8dU\xa8\x97\xee\"\x86\x8ete\xc8\x98\xdb\xd32\xc5\xe0\x12\x92\xd6\xc9\x950\xc4j=`\xe6jO\x0b\x84\x02\xa9e\x96\"\xf6\x94\xd0[p\x02J\xcdQ\xfa\xf1\xdc\x8e\xfdn\xcf>/\xc7\x10c.\xbe\xda\x8f2\x81S\xa7\xef\xaae\x0fDm\x985\x00T\xc0\x1fTCu\xeb\t\x13\xfa\xcb\x8f\x02\x01\x03\x02\x82\x01\x80vueA\xdf\xd2`L\x1bj\x99\x8e\x03\x1c\xe8\xfb\xf5\xbfvoq\x00\xac\xa7\xe7\x05{\xcaZ\xdcl(\xc3\xef=\xad\x18S\xf7\x1a\x98\x9b\xe4\xd5\'`\xb1\xb0\xb2g\xe7\x81\xee\xa2\xe2nW\x15\xe8H\x9a\xa6\xa8\xc8h\x7f\x9f\xd4MqE&\xcd\x86\xb3H?\x96\xda\xd0\xbc\xa4\xf1t\xde\x19\'y\x07B\xcb\x86\xe0\xb0v\x8bk.1p\xbe\xa3 \x1a\xa0(\xf8\xeetB\x148\xa5\xe0\xfdTX\xa6\xe4\x8e\xfe(\x9eH\x7f\x04\xbc\x7f)0\x92\x9eq\xc0\xda\xde\x1e\xa3\xa7\xe5\x05\x0b\x08xL\x87AZ\xf0\x11\x84F\xf3\xb6\x9d!\x88L\xc7\xd1\x1c\xe8L\xb2x\xc0\xdf\xb9\x905J\xa9\xaa\x08X\x9e\xebr\xb3\xcc\xc4e\xe7I\x1c\x0e\xce\x932\xa0R\xa1\xcc\x87[\x10\xc6\x8a\x81\x93\xe5\rm\x8eL{\x98\xe5\xe7\xfe\xa3%+\xbe*\xb9,\x94E\xfc\x9b\x03\xb1\x9b\xf6x_\xb6\x15\xbb\x1d\x9a\x01\xdf\xf9\xa0\x07\x86j\xbf\xc4\x1f\xf5\xda\xa6\x05\xccC\xc4.\xf5\x0b\x90\x1d\xc1\xecjj\x86\xaf\x90r\xb4\x80p\x8b\x10c\x90;\xcaD\x1aw$\x17t{\x0e\xe3VA\xfa\xe4w\xd2\xd89\x0e\'\xc3\xd1Y\xba&i\r\x8c\t\xe8\x86I\x84m3\xf7e\x02?\x05\xec\xc6c\xac5\x92\xd0t\xdb\xabSh\xc2\xca]\x02Z \x11\xd7\xd4.\xaa^\xaaG\xc2\xdb\x1b\x02?\x95E}\xf98\x80\xdcq\x10\xa0\x1flB\xeb\xf5v\x97\x84\x0e\xcb\xc2e\xa6\x8f\xa9@\x85]\xdb \xae\xb1\xe2\x0c\xaa\x1e\t*\xa7\"\x08\xf8\xab\x02\x81\xc1\x00\xe8\x95\x04;\x08\"0L(\x99\xf6\xdeY\x11\xe0\xe9\xde\xb4\xd7-(\xfe\xa0\x11\xf17eL\xe9\x81\xc7=\x84\xa9\x16\x8f\xb8\xfbk\x8e\xb7\xb8\xd7v\xbd^\x08\xd8ej\xe1\xdc&\x9d\xf5\xaa\x18\xfbv,\x16h~\xcaWY\x01#\xf2\x1f$\x160\xc7\x90\xa3\xd97\xdb\x05?L\xab<\xb1\x0e\xe2\xaa\xe85_d\x00\x0fJ\xb1\x00p\xa2\xd6q\xb1r\xe6C\xfd\x99\tV\x9b\xe3O\x1aD\xe6\x98o\x8bg\n\xfb\xa9\xf7\x93b\xfa\x17\x8c\x8f\x81M\xb7[E\xa1,\x97\xaan\x979\xab?f\xc3\xa6\xe9=f\x885\xe3\x07\x82\xae\xc5$\x9f\xd6\x92\x95\xa6$\xbb\xa2q\xd9]q\x04\xf5g\x96\xf4\xde\xbd=p!\xd7\x95e7\xa5\x19\xc6hQ\xd3A\x91\x81\x02\x81\xc1\x00\xc3\x94\"\xaf\xad\xe3*U\xe4?\xb9u\x93\x8f\xf0\xc3_\xee\xf7\xb0\x0b\xed\x13\x8dqV\"\xa99\x95\x17^y\xc6\x9d\xd1\xa9<\x92c2\xa3\x93\x97\xd2\xe7\xe5\x1a\xc9 mk\xf7\xfd\\5\xf4(8\xe6B5\xb5\x0ec\nB\xd09I\x80\x9b\x9a\x1d\x19\x14\x0bX2\x86\xef\x95\r\xeaW\xb4\xff\xa8:\xf6\xe8\x85i\xec\xbaw9T0\xca/T\xbd\xfe\x16=\xad\xc2w\xe3\xf8\x93\xa6<g\xac\xca\xc7\x88\xfe4\xb3\xca\xa3; 4#7\xcb\xcby\xff\xa1&\xae\xd1\x94|\x88\\\xe9\xb7\xc2hk\xdeA\x84\xe53\xda\xf4\x85\x1e\xed\xa0\x96wn\x0f\x05\xdc\x82\xea\x85o\xbe\xde\xab\x1a\x0e;\xf2O\xbf#_b\"\xb4\x87\xd4\xaf\xfc\xa1\xc2\xbc\x8d\xab\xc5\x0f\x02\x81\xc1\x00\x9b\x0e\x02\xd2\x05l 2\xc5\xbb\xf9\xe9\x90\xb6\x95\xf1?#:\x1e\x1bTj\xb6\xa0\xcf\x98\xdd\xf1\x01/~Xp\xb9\xb5%\xfc\xf2_%%\xe4\xf9\xd3\x94\x05\xe5\x98\xf1\xeb\xe8\x19\xbe\xa3\xc6\xbbRN\xc8\x0e\xf0T\x86\xe4\xe6\x00\xc2\xa1j\x18\x0e\xcb/\xb5\xc2\x90\xcf\xe7X\xd4\xdd\xc7}\xcb_A\xc7Ex\xeaB\xaa\xb4\xdcv\x00K\x179\xa1 \xf7D-S\xbb[\x8f\x12\x97\x8a\x11\x83De\x9f\xb2D\xb1\xfd\x1b\xfabA\xfc\x0f\xb3\nV3\xcf\x92.ks\x0f\xc6\xf4d\xd1\x1c\xd4\xef-\x19\xf0\xd3\x99\xb0#\xec\xaf\xact\x83m\xbf\xe4a\xb9\x19m\xd2lK\xe6>KX\xa3\x9ad\xa3?(\xd3\xa0\x16\x8f\xb8\xee%\x18\xbb\xd9\x9a\xe17\x81\x0b\xab\x02\x81\xc1\x00\x82b\xc1\xcas\xec\xc6\xe3\xed\x7f\xd0\xf9\r\n\xa0\x82?\xf4\xa5 \x07\xf3b^K\x8e\xc1\xc6&cd\xe9\xa6\x84i6p\xd3\x0cB!\xc2be7ECg0\xc0H\xf2\xa5S\x92\xce\xa2\xc5{D,#\xce\t\x97\\,\x8a\xd0\xdb\xab\x12f\xbe\x10\xb8\x07\x90!\xafJc^\x9c:x\xaap\'OE\xaeF\x9d\xd1\xa4\xd0\xe2\xcb1t\xe3)T\x0e\xd3\xc9,O\xedPbn\xd2\xef\xc8\x87/\xb0\xa9xw\xdcl\xd2\x15x\x17z\x87\xdc\xfb\xff\xc0\xc4t\x8b\xb8S\x05\x93Fz\x81\x9a\xf2\x94+\xad\xee\"\x91\xf8X\xbfI\x15\xb9\xa4\xf4\n\x03\xe8WG\x03\x9f\xd4\x94r\x11^\xd2\xa1\x8a\x7fl\xeaAlxZ\x8d\xca\xa8k\xd7(^r\x83_\x02\x81\xc03\x11n\xcb\x07\xb2\xc3:\xc7\xd8\xc7\xcc\xb4\xfb\xcaC\xe4p\xee\r\xe0\x0f\xde\xc8\x11\x07\x8c\x93\\SK\xe8c\x07\xb3\xa47c\x1clVM\x0b\xa1\x83\xb3L\xean\x9e\x85%A\xd0\xb1\xbb\xdc\xd1\x11\xe3}2\xefe}@|u\xfd\x8c\xb5\xf3\xf3\xc6dY\x18\x9d\xfd\xdf\xe0\x04\xf43\xfc\xa9\x90p\xb5\xd49zuC^MV\x91H\x1a\x17\xb7\xaf\\7\xcbH\t\x10\xb7\xd3\x94\x08\x016\xfb\x9f\xd0-\t\xcf\xb2\x13U\xe7W\x0f\xf6\x97\\\xdd\x9c\x95\x95C\xef\xe0\xf8\x07\xda\x8f\x15\xcc<\xec,\xe2\x8bfx\xba\x04\xfds\x10C\'\x07\xb6C=\xeat\x13>\xed\xb9iIM\x85Uu\xb5S\xab\xbd\xe1\xa5\xcd\x944k^\x85n\xab\xe7\xb7-\xd2\x02").unwrap();
            Ok(signer.sign::<_, Hasher>(&*key.rsa().unwrap())?)
        }
        pub fn dummy_signature(&mut self) -> &mut Self {
            self.signature = None;
            self
        }
        pub fn coresident_signature(&mut self) -> IoResult<&mut Self> {
            if let EnclaveSource::Path(path) = self.enclave {
                let sigfile = path.with_extension("sig");
                self.signature(sigfile)
            } else {
                Err(IoError::new(ErrorKind::NotFound,
                                 "Can\'t load coresident signature for non-file enclave"))
            }
        }
        pub fn signature<P: AsRef<Path>>(&mut self, path: P)
         -> IoResult<&mut Self> {
            let mut file = File::open(path)?;
            self.signature = Some(sigstruct::read(&mut file)?);
            Ok(self)
        }
        pub fn sigstruct(&mut self, sigstruct: Sigstruct) -> &mut Self {
            self.signature = Some(sigstruct);
            self
        }
        pub fn attributes(&mut self, attributes: Attributes) -> &mut Self {
            self.attributes = Some(attributes);
            self
        }
        pub fn miscselect(&mut self, miscselect: Miscselect) -> &mut Self {
            self.miscselect = Some(miscselect);
            self
        }
        fn load<T: Load>(mut self, loader: &mut T)
         -> Result<(Vec<ErasedTcs>, *mut c_void, usize), Error> {
            let signature =
                match self.signature {
                    Some(sig) => sig,
                    None =>
                    self.generate_dummy_signature().context("While generating dummy signature")?,
                };
            let attributes = self.attributes.unwrap_or(signature.attributes);
            let miscselect = self.miscselect.unwrap_or(signature.miscselect);
            let mapping =
                loader.load(&mut self.enclave, &signature, attributes,
                            miscselect)?;
            if mapping.tcss.is_empty() {
                {
                    ::std::rt::begin_panic("not yet implemented",
                                           &("enclave-runner/src/loader.rs",
                                             214u32, 13u32))
                }
            }
            Ok((mapping.tcss.into_iter().map(ErasedTcs::new).collect(),
                mapping.info.address(), mapping.info.size()))
        }
        pub fn build<T: Load>(self, loader: &mut T)
         -> Result<Command, Error> {
            self.load(loader).map(|(t, a, s)| Command::internal_new(t, a, s))
        }
        pub fn build_library<T: Load>(self, loader: &mut T)
         -> Result<Library, Error> {
            self.load(loader).map(|(t, a, s)| Library::internal_new(t, a, s))
        }
    }
}
mod tcs {
    use std;
    use std::cell::RefCell;
    use sgx_isa::Enclu;
    use sgxs::loader::Tcs;
    use crate::usercalls::abi::DispatchResult;
    use futures::prelude::*;
    use failure::Error;
    use crate::usercalls::abi::Register;
    use crate::usercalls::EnclaveAbort;
    use std::future::Future;
    pub(crate) type DebugBuffer = [u8; 1024];
    pub(crate) async fn enter<T: Tcs, F: 'static, R,
                              S: 'static>(tcs: T, mut state: S,
                                          mut on_usercall: F, p1: u64,
                                          p2: u64, p3: u64, p4: u64, p5: u64)
     -> Result<(T, S, DispatchResult), Error> where
     F: FnMut(&mut S, u64, u64, u64, u64, u64) -> R, R: Future<Output =
     Result<(Register, Register), EnclaveAbort<bool>>> {
        let mut result = coenter(tcs, p1, p2, p3, p4, p5);
        while let CoResult::Yield(usercall) = result {
            let (p1, p2, p3, p4, p5) = usercall.parameters();
            result =
                match {
                          #[allow(unused_imports)]
                          use ::tokio_async_await::compat::backward::IntoAwaitable
                              as IntoAwaitableBackward;
                          #[allow(unused_imports)]
                          use ::tokio_async_await::compat::forward::IntoAwaitable
                              as IntoAwaitableForward;
                          use ::tokio_async_await::std_await;
                          #[allow(unused_mut)]
                          let mut e =
                              on_usercall(&mut state, p1, p2, p3, p4, p5);
                          let e = e.into_awaitable();
                          {
                              let mut pinned = e;
                              loop  {
                                  if let ::std::task::Poll::Ready(x) =
                                         ::std::future::poll_with_tls_waker(unsafe
                                                                            {
                                                                                ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                            })
                                         {
                                      break x ;
                                  }
                                  yield
                              }
                          }
                      } {
                    Ok(ret) => usercall.coreturn(ret),
                    Err(err) => return Ok((usercall.tcs, state, Err(err))),
                }
        }
        match result {
            CoResult::Return((tcs, v1, v2)) => Ok((tcs, state, Ok((v1, v2)))),
            CoResult::Yield(_) => {
                {
                    ::std::rt::begin_panic("internal error: entered unreachable code",
                                           &("enclave-runner/src/tcs.rs",
                                             52u32, 31u32))
                }
            }
        }
    }
    pub enum CoResult<Y, R> { Yield(Y), Return(R), }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl <Y: ::std::fmt::Debug, R: ::std::fmt::Debug> ::std::fmt::Debug for
     CoResult<Y, R> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match (&*self,) {
                (&CoResult::Yield(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Yield");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&CoResult::Return(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Return");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    pub struct Usercall<T: Tcs> {
        tcs: T,
        parameters: (u64, u64, u64, u64, u64),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl <T: ::std::fmt::Debug + Tcs> ::std::fmt::Debug for Usercall<T> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                Usercall { tcs: ref __self_0_0, parameters: ref __self_0_1 }
                => {
                    let mut debug_trait_builder = f.debug_struct("Usercall");
                    let _ = debug_trait_builder.field("tcs", &&(*__self_0_0));
                    let _ =
                        debug_trait_builder.field("parameters",
                                                  &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    pub type ThreadResult<T> = CoResult<Usercall<T>, (T, u64, u64)>;
    impl <T: Tcs> Usercall<T> {
        pub fn parameters(&self) -> (u64, u64, u64, u64, u64) {
            self.parameters
        }
        pub fn coreturn(self, retval: (u64, u64)) -> ThreadResult<T> {
            coenter(self.tcs, 0, retval.0, retval.1, 0, 0)
        }
    }
    pub(crate) fn coenter<T: Tcs>(tcs: T, mut p1: u64, mut p2: u64,
                                  mut p3: u64, mut p4: u64, mut p5: u64)
     -> ThreadResult<T> {
        let sgx_result: u32;
        let mut _tmp: (u64, u64);
        unsafe {
            let debug_buf: Option<&RefCell<DebugBuffer>> = None;
            let mut uninit_debug_buf: DebugBuffer;
            let debug_buf = debug_buf.map(|r| r.borrow_mut());
            let debug_buf =
                match debug_buf {
                    Some(mut buf) => buf.as_mut_ptr(),
                    None => {
                        uninit_debug_buf = std::mem::uninitialized();
                        uninit_debug_buf.as_mut_ptr()
                    }
                };
            asm!("\n\t\tlea 1f(%rip),%rcx\n1:\n\t\tenclu\n":
                "={eax}"(sgx_result), "={rbx}"(_tmp.0), "={r10}"(_tmp.1),
                "={rdi}"(p1), "={rsi}"(p2), "={rdx}"(p3), "={r8}"(p4),
                "={r9}"(p5) :
                "{eax}"(2), "{rbx}"(tcs.address()), "{r10}"(debug_buf),
                "{rdi}"(p1), "{rsi}"(p2), "{rdx}"(p3), "{r8}"(p4), "{r9}"(p5)
                : "rcx", "r11", "memory" : "volatile")
        };
        if sgx_result != (Enclu::EExit as u32) {
            {
                ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["Invalid return value in EAX! eax="],
                                                                          &match (&sgx_result,)
                                                                               {
                                                                               (arg0,)
                                                                               =>
                                                                               [::std::fmt::ArgumentV1::new(arg0,
                                                                                                            ::std::fmt::Display::fmt)],
                                                                           }),
                                           &("enclave-runner/src/tcs.rs",
                                             122u32, 9u32))
            };
        }
        if p1 == 0 {
            CoResult::Return((tcs, p2, p3))
        } else {
            CoResult::Yield(Usercall{tcs: tcs,
                                     parameters: (p1, p2, p3, p4, p5),})
        }
    }
}
mod usercalls {
    extern crate libc;
    extern crate nix;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::fmt;
    use std::io::{self, ErrorKind as IoErrorKind, Read, Result as IoResult,
                  Write};
    use std::net::{TcpListener, TcpStream};
    use std::result::Result as StdResult;
    use std::str;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc::{self, channel, Receiver, RecvError, Sender,
                          sync_channel, SyncSender};
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time;
    use failure;
    use fnv::FnvHashMap;
    use fortanix_sgx_abi::*;
    use sgxs::loader::Tcs as SgxsTcs;
    use futures::prelude::*;
    use tokio::prelude::*;
    use futures::future::lazy;
    use futures::future::Future;
    #[allow(missing_copy_implementations)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct DEBUGGER_TOGGLE_SYNC {
        __private_field: (),
    }
    #[doc(hidden)]
    static DEBUGGER_TOGGLE_SYNC: DEBUGGER_TOGGLE_SYNC =
        DEBUGGER_TOGGLE_SYNC{__private_field: (),};
    impl ::lazy_static::__Deref for DEBUGGER_TOGGLE_SYNC {
        type
        Target
        =
        Mutex<()>;
        fn deref(&self) -> &Mutex<()> {
            #[inline(always)]
            fn __static_ref_initialize() -> Mutex<()> { Mutex::new(()) }
            #[inline(always)]
            fn __stability() -> &'static Mutex<()> {
                static LAZY: ::lazy_static::lazy::Lazy<Mutex<()>> =
                    ::lazy_static::lazy::Lazy::INIT;
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
    impl ::lazy_static::LazyStatic for DEBUGGER_TOGGLE_SYNC {
        fn initialize(lazy: &Self) { let _ = &**lazy; }
    }
    pub mod abi {
        //! Trait-based usercall dispatching based on the ABI specification.
        //!
        //! The macros in this module implement a `trait Usercalls` and a `fn dispatch`
        //! that together implement usercall handling according to the parsed ABI.
        #![allow(unused)]
        use fortanix_sgx_abi::*;
        use std::ptr::NonNull;
        use std::sync::atomic::{AtomicUsize, Ordering};
        pub type Register = u64;
        trait RegisterArgument {
            fn from_register(a: Register)
            -> Self;
            fn into_register(self)
            -> Register;
        }
        type EnclaveAbort = super::EnclaveAbort<bool>;
        pub(crate) type UsercallResult<T>
            =
            ::std::result::Result<T, EnclaveAbort>;
        pub(crate) type DispatchResult = UsercallResult<(Register, Register)>;
        trait ReturnValue {
            fn into_registers(self)
            -> DispatchResult;
        }
        macro_rules! define_usercalls((
                                      $ (
                                      fn $ f : ident (
                                      $ ( $ n : ident : $ t : ty ) , * ) $ (
                                      -> $ r : tt ) * ; ) * ) => {
                                      # [ repr ( C ) ] # [
                                      allow ( non_camel_case_types ) ] enum
                                      UsercallList {
                                      __enclave_usercalls_invalid , $ ( $ f ,
                                      ) * } pub ( super ) trait Usercalls {
                                      $ (
                                      fn $ f (
                                      & mut self , $ ( $ n : $ t ) , * ) ->
                                      dispatch_return_type ! ( $ ( -> $ r ) *
                                      ) ; ) * fn other (
                                      & mut self , n : u64 , a1 : u64 , a2 :
                                      u64 , a3 : u64 , a4 : u64 ) ->
                                      DispatchResult {
                                      Err (
                                      $ crate :: usercalls :: EnclaveAbort ::
                                      InvalidUsercall ( n ) ) } fn is_exiting
                                      ( & self ) -> bool ; } # [
                                      allow ( unused_variables ) ] pub ( super
                                      ) fn dispatch < H : Usercalls > (
                                      handler : & mut H , n : u64 , a1 : u64 ,
                                      a2 : u64 , a3 : u64 , a4 : u64 ) ->
                                      DispatchResult {
                                      let ret = $ (
                                      if n == UsercallList :: $ f as Register
                                      {
                                      ReturnValue :: into_registers (
                                      unsafe {
                                      enclave_usercalls_internal_define_usercalls
                                      ! (
                                      handler , replace_args a1 , a2 , a3 , a4
                                      $ f ( $ ( $ n ) , * ) ) } ) } else ) * {
                                      handler . other ( n , a1 , a2 , a3 , a4
                                      ) } ; if ret . is_ok (  ) && handler .
                                      is_exiting (  ) {
                                      Err ( super :: EnclaveAbort :: Secondary
                                      ) } else { ret } } } ;);
        macro_rules! define_ra(( < $ i : ident > $ t : ty ) => {
                               impl < $ i > RegisterArgument for $ t {
                               fn from_register ( a : Register ) -> Self {
                               a as _ } fn into_register ( self ) -> Register
                               { self as _ } } } ; ( $ i : ty as $ t : ty ) =>
                               {
                               impl RegisterArgument for $ t {
                               fn from_register ( a : Register ) -> Self {
                               a as $ i as _ } fn into_register ( self ) ->
                               Register { self as $ i as _ } } } ; ( $ t : ty
                               ) => {
                               impl RegisterArgument for $ t {
                               fn from_register ( a : Register ) -> Self {
                               a as _ } fn into_register ( self ) -> Register
                               { self as _ } } } ;);
        impl RegisterArgument for Register {
            fn from_register(a: Register) -> Self { a as _ }
            fn into_register(self) -> Register { self as _ }
        }
        impl RegisterArgument for i64 {
            fn from_register(a: Register) -> Self { a as _ }
            fn into_register(self) -> Register { self as _ }
        }
        impl RegisterArgument for u32 {
            fn from_register(a: Register) -> Self { a as _ }
            fn into_register(self) -> Register { self as _ }
        }
        impl RegisterArgument for i32 {
            fn from_register(a: Register) -> Self { a as u32 as _ }
            fn into_register(self) -> Register { self as u32 as _ }
        }
        impl RegisterArgument for u16 {
            fn from_register(a: Register) -> Self { a as _ }
            fn into_register(self) -> Register { self as _ }
        }
        impl RegisterArgument for i16 {
            fn from_register(a: Register) -> Self { a as u16 as _ }
            fn into_register(self) -> Register { self as u16 as _ }
        }
        impl RegisterArgument for u8 {
            fn from_register(a: Register) -> Self { a as _ }
            fn into_register(self) -> Register { self as _ }
        }
        impl RegisterArgument for i8 {
            fn from_register(a: Register) -> Self { a as u8 as _ }
            fn into_register(self) -> Register { self as u8 as _ }
        }
        impl RegisterArgument for usize {
            fn from_register(a: Register) -> Self { a as _ }
            fn into_register(self) -> Register { self as _ }
        }
        impl RegisterArgument for isize {
            fn from_register(a: Register) -> Self { a as usize as _ }
            fn into_register(self) -> Register { self as usize as _ }
        }
        impl <T> RegisterArgument for *const T {
            fn from_register(a: Register) -> Self { a as _ }
            fn into_register(self) -> Register { self as _ }
        }
        impl <T> RegisterArgument for *mut T {
            fn from_register(a: Register) -> Self { a as _ }
            fn into_register(self) -> Register { self as _ }
        }
        impl RegisterArgument for () {
            fn from_register(_: Register) -> () { () }
            fn into_register(self) -> Register { 0 }
        }
        impl RegisterArgument for bool {
            fn from_register(a: Register) -> bool {
                if a != 0 { true } else { false }
            }
            fn into_register(self) -> Register { self as _ }
        }
        impl <T: RegisterArgument> RegisterArgument for Option<NonNull<T>> {
            fn from_register(a: Register) -> Option<NonNull<T>> {
                NonNull::new(a as _)
            }
            fn into_register(self) -> Register {
                self.map_or(0 as _, NonNull::as_ptr) as _
            }
        }
        impl ReturnValue for EnclaveAbort {
            fn into_registers(self) -> DispatchResult { Err(self) }
        }
        impl <T: RegisterArgument> ReturnValue for UsercallResult<T> {
            fn into_registers(self) -> DispatchResult {
                self.map(|v| (v.into_register(), 0))
            }
        }
        impl <T: RegisterArgument, U: RegisterArgument> ReturnValue for
         UsercallResult<(T, U)> {
            fn into_registers(self) -> DispatchResult {
                self.map(|(a, b)| (a.into_register(), b.into_register()))
            }
        }
        macro_rules! dispatch_return_type(( -> ! ) => { EnclaveAbort } ; (
                                          -> $ r : ty ) => {
                                          UsercallResult < $ r > } ; (  ) => {
                                          UsercallResult < (  ) > } ;);
        macro_rules! enclave_usercalls_internal_define_usercalls((
                                                                 $ h : ident ,
                                                                 replace_args
                                                                 $ a1 : ident
                                                                 , $ a2 :
                                                                 ident , $ a3
                                                                 : ident , $
                                                                 a4 : ident $
                                                                 f : ident (
                                                                 $ n1 : ident
                                                                 , $ n2 :
                                                                 ident , $ n3
                                                                 : ident , $
                                                                 n4 : ident )
                                                                 ) => {
                                                                 H :: $ f (
                                                                 $ h ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a1 ) ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a2 ) ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a3 ) ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a4 ) , )
                                                                 } ; (
                                                                 $ h : ident ,
                                                                 replace_args
                                                                 $ a1 : ident
                                                                 , $ a2 :
                                                                 ident , $ a3
                                                                 : ident , $
                                                                 a4 : ident $
                                                                 f : ident (
                                                                 $ n1 : ident
                                                                 , $ n2 :
                                                                 ident , $ n3
                                                                 : ident ) )
                                                                 => {
                                                                 {
                                                                 assert_eq ! (
                                                                 $ a4 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "4th" ) ; H
                                                                 :: $ f (
                                                                 $ h ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a1 ) ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a2 ) ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a3 ) , )
                                                                 } } ; (
                                                                 $ h : ident ,
                                                                 replace_args
                                                                 $ a1 : ident
                                                                 , $ a2 :
                                                                 ident , $ a3
                                                                 : ident , $
                                                                 a4 : ident $
                                                                 f : ident (
                                                                 $ n1 : ident
                                                                 , $ n2 :
                                                                 ident ) ) =>
                                                                 {
                                                                 {
                                                                 assert_eq ! (
                                                                 $ a3 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "3rd" ) ;
                                                                 assert_eq ! (
                                                                 $ a4 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "4th" ) ; H
                                                                 :: $ f (
                                                                 $ h ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a1 ) ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a2 ) , )
                                                                 } } ; (
                                                                 $ h : ident ,
                                                                 replace_args
                                                                 $ a1 : ident
                                                                 , $ a2 :
                                                                 ident , $ a3
                                                                 : ident , $
                                                                 a4 : ident $
                                                                 f : ident (
                                                                 $ n1 : ident
                                                                 ) ) => {
                                                                 {
                                                                 assert_eq ! (
                                                                 $ a2 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "2nd" ) ;
                                                                 assert_eq ! (
                                                                 $ a3 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "3rd" ) ;
                                                                 assert_eq ! (
                                                                 $ a4 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "4th" ) ; H
                                                                 :: $ f (
                                                                 $ h ,
                                                                 RegisterArgument
                                                                 ::
                                                                 from_register
                                                                 ( $ a1 ) ) }
                                                                 } ; (
                                                                 $ h : ident ,
                                                                 replace_args
                                                                 $ a1 : ident
                                                                 , $ a2 :
                                                                 ident , $ a3
                                                                 : ident , $
                                                                 a4 : ident $
                                                                 f : ident (
                                                                 ) ) => {
                                                                 {
                                                                 assert_eq ! (
                                                                 $ a1 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "1st" ) ;
                                                                 assert_eq ! (
                                                                 $ a2 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "2nd" ) ;
                                                                 assert_eq ! (
                                                                 $ a3 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "3rd" ) ;
                                                                 assert_eq ! (
                                                                 $ a4 , 0 ,
                                                                 "Usercall {}: expected {} argument to be 0"
                                                                 , stringify !
                                                                 ( $ f ) ,
                                                                 "4th" ) ; H
                                                                 :: $ f ( $ h
                                                                 ) } } ;);
        #[repr(C)]
        #[allow(non_camel_case_types)]
        enum UsercallList {
            __enclave_usercalls_invalid,
            read,
            read_alloc,
            write,
            flush,
            close,
            bind_stream,
            accept_stream,
            connect_stream,
            launch_thread,
            exit,
            wait,
            send,
            insecure_time,
            alloc,
            free,
            async_queues,
        }
        pub(super) trait Usercalls {
            fn read(&mut self, fd: Fd, buf: *mut u8, len: usize)
            -> UsercallResult<(Result, usize)>;
            fn read_alloc(&mut self, fd: Fd, buf: *mut ByteBuffer)
            -> UsercallResult<Result>;
            fn write(&mut self, fd: Fd, buf: *const u8, len: usize)
            -> UsercallResult<(Result, usize)>;
            fn flush(&mut self, fd: Fd)
            -> UsercallResult<Result>;
            fn close(&mut self, fd: Fd)
            -> UsercallResult<()>;
            fn bind_stream(&mut self, addr: *const u8, len: usize,
                           local_addr: *mut ByteBuffer)
            -> UsercallResult<(Result, Fd)>;
            fn accept_stream(&mut self, fd: Fd, local_addr: *mut ByteBuffer,
                             peer_addr: *mut ByteBuffer)
            -> UsercallResult<(Result, Fd)>;
            fn connect_stream(&mut self, addr: *const u8, len: usize,
                              local_addr: *mut ByteBuffer,
                              peer_addr: *mut ByteBuffer)
            -> UsercallResult<(Result, Fd)>;
            fn launch_thread(&mut self)
            -> UsercallResult<Result>;
            fn exit(&mut self, panic: bool)
            -> EnclaveAbort;
            fn wait(&mut self, event_mask: u64, timeout: u64)
            -> UsercallResult<(Result, u64)>;
            fn send(&mut self, event_set: u64, tcs: Option<Tcs>)
            -> UsercallResult<Result>;
            fn insecure_time(&mut self)
            -> UsercallResult<u64>;
            fn alloc(&mut self, size: usize, alignment: usize)
            -> UsercallResult<(Result, *mut u8)>;
            fn free(&mut self, ptr: *mut u8, size: usize, alignment: usize)
            -> UsercallResult<()>;
            fn async_queues(&mut self,
                            usercall_queue: *mut FifoDescriptor<Usercall>,
                            return_queue: *mut FifoDescriptor<Return>)
            -> UsercallResult<Result>;
            fn other(&mut self, n: u64, a1: u64, a2: u64, a3: u64, a4: u64)
             -> DispatchResult {
                Err(crate::usercalls::EnclaveAbort::InvalidUsercall(n))
            }
            fn is_exiting(&self)
            -> bool;
        }
        #[allow(unused_variables)]
        pub(super) fn dispatch<H: Usercalls>(handler: &mut H, n: u64, a1: u64,
                                             a2: u64, a3: u64, a4: u64)
         -> DispatchResult {
            let ret =
                if n == UsercallList::read as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"read",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::read(handler,
                                                                RegisterArgument::from_register(a1),
                                                                RegisterArgument::from_register(a2),
                                                                RegisterArgument::from_register(a3))
                                                    }
                                                })
                } else if n == UsercallList::read_alloc as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"read_alloc",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"read_alloc",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::read_alloc(handler,
                                                                      RegisterArgument::from_register(a1),
                                                                      RegisterArgument::from_register(a2))
                                                    }
                                                })
                } else if n == UsercallList::write as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"write",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::write(handler,
                                                                 RegisterArgument::from_register(a1),
                                                                 RegisterArgument::from_register(a2),
                                                                 RegisterArgument::from_register(a3))
                                                    }
                                                })
                } else if n == UsercallList::flush as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a2),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"flush",
                                                                                                                                                                                     &"2nd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"flush",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"flush",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::flush(handler,
                                                                 RegisterArgument::from_register(a1))
                                                    }
                                                })
                } else if n == UsercallList::close as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a2),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"close",
                                                                                                                                                                                     &"2nd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"close",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"close",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::close(handler,
                                                                 RegisterArgument::from_register(a1))
                                                    }
                                                })
                } else if n == UsercallList::bind_stream as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"bind_stream",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::bind_stream(handler,
                                                                       RegisterArgument::from_register(a1),
                                                                       RegisterArgument::from_register(a2),
                                                                       RegisterArgument::from_register(a3))
                                                    }
                                                })
                } else if n == UsercallList::accept_stream as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"accept_stream",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::accept_stream(handler,
                                                                         RegisterArgument::from_register(a1),
                                                                         RegisterArgument::from_register(a2),
                                                                         RegisterArgument::from_register(a3))
                                                    }
                                                })
                } else if n == UsercallList::connect_stream as Register {
                    ReturnValue::into_registers(unsafe {
                                                    H::connect_stream(handler,
                                                                      RegisterArgument::from_register(a1),
                                                                      RegisterArgument::from_register(a2),
                                                                      RegisterArgument::from_register(a3),
                                                                      RegisterArgument::from_register(a4))
                                                })
                } else if n == UsercallList::launch_thread as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a1),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"launch_thread",
                                                                                                                                                                                     &"1st")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a2),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"launch_thread",
                                                                                                                                                                                     &"2nd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"launch_thread",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"launch_thread",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::launch_thread(handler)
                                                    }
                                                })
                } else if n == UsercallList::exit as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a2),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"exit",
                                                                                                                                                                                     &"2nd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"exit",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"exit",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::exit(handler,
                                                                RegisterArgument::from_register(a1))
                                                    }
                                                })
                } else if n == UsercallList::wait as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"wait",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"wait",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::wait(handler,
                                                                RegisterArgument::from_register(a1),
                                                                RegisterArgument::from_register(a2))
                                                    }
                                                })
                } else if n == UsercallList::send as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"send",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"send",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::send(handler,
                                                                RegisterArgument::from_register(a1),
                                                                RegisterArgument::from_register(a2))
                                                    }
                                                })
                } else if n == UsercallList::insecure_time as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a1),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"insecure_time",
                                                                                                                                                                                     &"1st")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a2),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"insecure_time",
                                                                                                                                                                                     &"2nd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"insecure_time",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"insecure_time",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::insecure_time(handler)
                                                    }
                                                })
                } else if n == UsercallList::alloc as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"alloc",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"alloc",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::alloc(handler,
                                                                 RegisterArgument::from_register(a1),
                                                                 RegisterArgument::from_register(a2))
                                                    }
                                                })
                } else if n == UsercallList::free as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"free",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::free(handler,
                                                                RegisterArgument::from_register(a1),
                                                                RegisterArgument::from_register(a2),
                                                                RegisterArgument::from_register(a3))
                                                    }
                                                })
                } else if n == UsercallList::async_queues as Register {
                    ReturnValue::into_registers(unsafe {
                                                    {
                                                        {
                                                            match (&(a3),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"async_queues",
                                                                                                                                                                                     &"3rd")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        {
                                                            match (&(a4),
                                                                   &(0)) {
                                                                (left_val,
                                                                 right_val) =>
                                                                {
                                                                    if !(*left_val
                                                                             ==
                                                                             *right_val)
                                                                       {
                                                                        {
                                                                            ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                                                        "`,\n right: `",
                                                                                                                                        "`: "],
                                                                                                                                      &match (&&*left_val,
                                                                                                                                              &&*right_val,
                                                                                                                                              &::std::fmt::Arguments::new_v1(&["Usercall ",
                                                                                                                                                                               ": expected ",
                                                                                                                                                                               " argument to be 0"],
                                                                                                                                                                             &match (&"async_queues",
                                                                                                                                                                                     &"4th")
                                                                                                                                                                                  {
                                                                                                                                                                                  (arg0,
                                                                                                                                                                                   arg1)
                                                                                                                                                                                  =>
                                                                                                                                                                                  [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                                               ::std::fmt::Display::fmt),
                                                                                                                                                                                   ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                                                               ::std::fmt::Display::fmt)],
                                                                                                                                                                              }))
                                                                                                                                           {
                                                                                                                                           (arg0,
                                                                                                                                            arg1,
                                                                                                                                            arg2)
                                                                                                                                           =>
                                                                                                                                           [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                        ::std::fmt::Debug::fmt),
                                                                                                                                            ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                                                        ::std::fmt::Display::fmt)],
                                                                                                                                       }),
                                                                                                       &("enclave-runner/src/usercalls/abi.rs",
                                                                                                         299u32,
                                                                                                         1u32))
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        };
                                                        H::async_queues(handler,
                                                                        RegisterArgument::from_register(a1),
                                                                        RegisterArgument::from_register(a2))
                                                    }
                                                })
                } else { handler.other(n, a1, a2, a3, a4) };
            if ret.is_ok() && handler.is_exiting() {
                Err(super::EnclaveAbort::Secondary)
            } else { ret }
        }
    }
    mod interface {
        //! Adaptors between the usercall ABI types and functions and (mostly) safe
        //! Rust types.
        use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as
                      IoResult};
        use std::slice;
        use fortanix_sgx_abi::*;
        use super::abi::{UsercallResult, Usercalls};
        use super::{EnclaveAbort, RunningTcs};
        pub(super) struct Handler<'a>(pub &'a mut RunningTcs);
        impl <'a> Usercalls for Handler<'a> {
            fn is_exiting(&self) -> bool { self.0.is_exiting() }
            fn read(&mut self, fd: Fd, buf: *mut u8, len: usize)
             -> UsercallResult<(Result, usize)> {
                unsafe {
                    Ok(from_raw_parts_mut_nonnull(buf,
                                                  len).and_then(|buf|
                                                                    self.0.read(fd,
                                                                                buf)).to_sgx_result())
                }
            }
            fn read_alloc(&mut self, fd: Fd, buf: *mut ByteBuffer)
             -> UsercallResult<Result> {
                unsafe {
                    Ok((||
                            {
                                let mut out =
                                    OutputBuffer::new(buf.as_mut().ok_or(IoErrorKind::InvalidInput)?);
                                if !out.buf.data.is_null() {
                                    return Err(IoErrorKind::InvalidInput.into());
                                }
                                self.0.read_alloc(fd, &mut out)
                            })().to_sgx_result())
                }
            }
            fn write(&mut self, fd: Fd, buf: *const u8, len: usize)
             -> UsercallResult<(Result, usize)> {
                unsafe {
                    Ok(from_raw_parts_nonnull(buf,
                                              len).and_then(|buf|
                                                                self.0.write(fd,
                                                                             buf)).to_sgx_result())
                }
            }
            fn flush(&mut self, fd: Fd) -> UsercallResult<Result> {
                Ok(self.0.flush(fd).to_sgx_result())
            }
            fn close(&mut self, fd: Fd) -> UsercallResult<()> {
                Ok(self.0.close(fd))
            }
            fn bind_stream(&mut self, addr: *const u8, len: usize,
                           local_addr: *mut ByteBuffer)
             -> UsercallResult<(Result, Fd)> {
                unsafe {
                    let mut local_addr =
                        local_addr.as_mut().map(OutputBuffer::new);
                    Ok(from_raw_parts_nonnull(addr,
                                              len).and_then(|addr|
                                                                self.0.bind_stream(addr,
                                                                                   local_addr.as_mut())).to_sgx_result())
                }
            }
            fn accept_stream(&mut self, fd: Fd, local_addr: *mut ByteBuffer,
                             peer_addr: *mut ByteBuffer)
             -> UsercallResult<(Result, Fd)> {
                unsafe {
                    let mut local_addr =
                        local_addr.as_mut().map(OutputBuffer::new);
                    let mut peer_addr =
                        peer_addr.as_mut().map(OutputBuffer::new);
                    Ok(self.0.accept_stream(fd, local_addr.as_mut(),
                                            peer_addr.as_mut()).to_sgx_result())
                }
            }
            fn connect_stream(&mut self, addr: *const u8, len: usize,
                              local_addr: *mut ByteBuffer,
                              peer_addr: *mut ByteBuffer)
             -> UsercallResult<(Result, Fd)> {
                unsafe {
                    let mut local_addr =
                        local_addr.as_mut().map(OutputBuffer::new);
                    let mut peer_addr =
                        peer_addr.as_mut().map(OutputBuffer::new);
                    Ok(from_raw_parts_nonnull(addr,
                                              len).and_then(|addr|
                                                                {
                                                                    self.0.connect_stream(addr,
                                                                                          local_addr.as_mut(),
                                                                                          peer_addr.as_mut())
                                                                }).to_sgx_result())
                }
            }
            fn launch_thread(&mut self) -> UsercallResult<Result> {
                Ok(self.0.launch_thread().to_sgx_result())
            }
            fn exit(&mut self, panic: bool) -> EnclaveAbort<bool> {
                self.0.exit(panic)
            }
            fn wait(&mut self, event_mask: u64, timeout: u64)
             -> UsercallResult<(Result, u64)> {
                if event_mask == 0 && timeout == WAIT_INDEFINITE {
                    return Err(EnclaveAbort::IndefiniteWait);
                }
                Ok(self.0.wait(event_mask, timeout).to_sgx_result())
            }
            fn send(&mut self, event_set: u64, tcs: Option<Tcs>)
             -> UsercallResult<Result> {
                Ok(self.0.send(event_set, tcs).to_sgx_result())
            }
            fn insecure_time(&mut self) -> UsercallResult<u64> {
                Ok(self.0.insecure_time())
            }
            fn alloc(&mut self, size: usize, alignment: usize)
             -> UsercallResult<(Result, *mut u8)> {
                Ok(self.0.alloc(size, alignment).to_sgx_result())
            }
            fn free(&mut self, ptr: *mut u8, size: usize, alignment: usize)
             -> UsercallResult<()> {
                Ok(self.0.free(ptr, size, alignment).unwrap())
            }
            fn async_queues(&mut self,
                            usercall_queue: *mut FifoDescriptor<Usercall>,
                            return_queue: *mut FifoDescriptor<Return>)
             -> UsercallResult<Result> {
                unsafe {
                    Ok((||
                            {
                                let usercall_queue =
                                    usercall_queue.as_mut().ok_or(IoError::from(IoErrorKind::InvalidInput))?;
                                let return_queue =
                                    return_queue.as_mut().ok_or(IoError::from(IoErrorKind::InvalidInput))?;
                                self.0.async_queues(usercall_queue,
                                                    return_queue)
                            })().to_sgx_result())
                }
            }
        }
        pub(super) struct OutputBuffer<'a> {
            buf: &'a mut ByteBuffer,
            data: Option<Box<[u8]>>,
        }
        impl <'a> OutputBuffer<'a> {
            fn new(buf: &'a mut ByteBuffer) -> Self {
                OutputBuffer{buf, data: None,}
            }
            pub(super) fn set<T: Into<Box<[u8]>>>(&mut self, value: T) {
                self.data = Some(value.into());
            }
        }
        impl <'a> Drop for OutputBuffer<'a> {
            fn drop(&mut self) {
                if let Some(buf) = self.data.take() {
                    self.buf.len = buf.len();
                    self.buf.data = Box::into_raw(buf) as _;
                } else { self.buf.len = 0; }
            }
        }
        fn result_from_io_error(err: IoError) -> Result {
            let ret =
                match err.kind() {
                    IoErrorKind::NotFound => Error::NotFound,
                    IoErrorKind::PermissionDenied => Error::PermissionDenied,
                    IoErrorKind::ConnectionRefused =>
                    Error::ConnectionRefused,
                    IoErrorKind::ConnectionReset => Error::ConnectionReset,
                    IoErrorKind::ConnectionAborted =>
                    Error::ConnectionAborted,
                    IoErrorKind::NotConnected => Error::NotConnected,
                    IoErrorKind::AddrInUse => Error::AddrInUse,
                    IoErrorKind::AddrNotAvailable => Error::AddrNotAvailable,
                    IoErrorKind::BrokenPipe => Error::BrokenPipe,
                    IoErrorKind::AlreadyExists => Error::AlreadyExists,
                    IoErrorKind::WouldBlock => Error::WouldBlock,
                    IoErrorKind::InvalidInput => Error::InvalidInput,
                    IoErrorKind::InvalidData => Error::InvalidData,
                    IoErrorKind::TimedOut => Error::TimedOut,
                    IoErrorKind::WriteZero => Error::WriteZero,
                    IoErrorKind::Interrupted => Error::Interrupted,
                    IoErrorKind::Other => Error::Other,
                    IoErrorKind::UnexpectedEof => Error::UnexpectedEof,
                    _ => Error::Other,
                };
            ret as _
        }
        trait ToSgxResult {
            type
            Return;
            fn to_sgx_result(self)
            -> Self::Return;
        }
        trait SgxReturn {
            fn on_error()
            -> Self;
        }
        impl SgxReturn for u64 {
            fn on_error() -> Self { 0 }
        }
        impl SgxReturn for usize {
            fn on_error() -> Self { 0 }
        }
        impl SgxReturn for *mut u8 {
            fn on_error() -> Self { ::std::ptr::null_mut() }
        }
        impl <T: SgxReturn> ToSgxResult for IoResult<T> {
            type
            Return
            =
            (Result, T);
            fn to_sgx_result(self) -> Self::Return {
                match self {
                    Err(e) => (result_from_io_error(e), T::on_error()),
                    Ok(v) => (RESULT_SUCCESS, v),
                }
            }
        }
        impl ToSgxResult for IoResult<()> {
            type
            Return
            =
            Result;
            fn to_sgx_result(self) -> Self::Return {
                self.err().map_or(RESULT_SUCCESS, |e| result_from_io_error(e))
            }
        }
        pub unsafe fn from_raw_parts_nonnull<'a, T>(p: *const T, len: usize)
         -> IoResult<&'a [T]> {
            if p.is_null() {
                Err(IoErrorKind::InvalidInput.into())
            } else { Ok(slice::from_raw_parts(p, len)) }
        }
        pub unsafe fn from_raw_parts_mut_nonnull<'a, T>(p: *mut T, len: usize)
         -> IoResult<&'a mut [T]> {
            if p.is_null() {
                Err(IoErrorKind::InvalidInput.into())
            } else { Ok(slice::from_raw_parts_mut(p, len)) }
        }
    }
    use self::abi::dispatch;
    use self::interface::{Handler, OutputBuffer};
    use self::libc::*;
    use self::nix::sys::signal;
    use crate::loader::{EnclavePanic, ErasedTcs};
    use crate::tcs;
    use crate::usercalls::abi::Register;
    use crate::usercalls::abi::DispatchResult;
    const EV_ABORT: u64 = 8;
    struct ReadOnly<R>(R);
    struct WriteOnly<W>(W);
    macro_rules! forward((
                         fn $ n : ident (
                         & mut self $ ( , $ p : ident : $ t : ty ) * ) -> $
                         ret : ty ) => {
                         fn $ n ( & mut self $ ( , $ p : $ t ) * ) -> $ ret {
                         self . 0 . $ n ( $ ( $ p ) , * ) } });
    pub struct GlobalSyncSender {
        mutex: Arc<Mutex<(Option<SyncSender<i32>>)>>,
    }
    #[allow(missing_copy_implementations)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct ThreadSyncSender {
        __private_field: (),
    }
    #[doc(hidden)]
    static ThreadSyncSender: ThreadSyncSender =
        ThreadSyncSender{__private_field: (),};
    impl ::lazy_static::__Deref for ThreadSyncSender {
        type
        Target
        =
        GlobalSyncSender;
        fn deref(&self) -> &GlobalSyncSender {
            #[inline(always)]
            fn __static_ref_initialize() -> GlobalSyncSender {
                GlobalSyncSender{mutex: Arc::new(Mutex::new(None)),}
            }
            #[inline(always)]
            fn __stability() -> &'static GlobalSyncSender {
                static LAZY: ::lazy_static::lazy::Lazy<GlobalSyncSender> =
                    ::lazy_static::lazy::Lazy::INIT;
                LAZY.get(__static_ref_initialize)
            }
            __stability()
        }
    }
    impl ::lazy_static::LazyStatic for ThreadSyncSender {
        fn initialize(lazy: &Self) { let _ = &**lazy; }
    }
    impl <R: Read> Read for ReadOnly<R> {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            self.0.read(buf)
        }
    }
    impl <T> Read for WriteOnly<T> {
        fn read(&mut self, _buf: &mut [u8]) -> IoResult<usize> {
            Err(IoErrorKind::BrokenPipe.into())
        }
    }
    impl <T> Write for ReadOnly<T> {
        fn write(&mut self, _buf: &[u8]) -> IoResult<usize> {
            Err(IoErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> IoResult<()> {
            Err(IoErrorKind::BrokenPipe.into())
        }
    }
    impl <W: Write> Write for WriteOnly<W> {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.0.write(buf)
        }
        fn flush(&mut self) -> IoResult<()> { self.0.flush() }
    }
    trait SharedStream<'a> {
        type
        Inner: Read +
        Write +
        'a;
        fn lock(&'a self)
        -> Self::Inner;
    }
    struct Shared<T>(T);
    impl <'a, T: SharedStream<'a>> Read for &'a Shared<T> {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            self.0.lock().read(buf)
        }
    }
    impl <'a, T: SharedStream<'a>> Write for &'a Shared<T> {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.0.lock().write(buf)
        }
        fn flush(&mut self) -> IoResult<()> { self.0.lock().flush() }
    }
    impl <'a> SharedStream<'a> for io::Stdin {
        type
        Inner
        =
        ReadOnly<io::StdinLock<'a>>;
        fn lock(&'a self) -> Self::Inner { ReadOnly(io::Stdin::lock(self)) }
    }
    impl <'a> SharedStream<'a> for io::Stdout {
        type
        Inner
        =
        WriteOnly<io::StdoutLock<'a>>;
        fn lock(&'a self) -> Self::Inner { WriteOnly(io::Stdout::lock(self)) }
    }
    impl <'a> SharedStream<'a> for io::Stderr {
        type
        Inner
        =
        WriteOnly<io::StderrLock<'a>>;
        fn lock(&'a self) -> Self::Inner { WriteOnly(io::Stderr::lock(self)) }
    }
    impl <S: 'static + Send + Sync> SyncStream for S where
     for<'a> &'a S: Read + Write {
        fn read(&self, buf: &mut [u8]) -> IoResult<usize> {
            Read::read(&mut { self }, buf)
        }
        fn write(&self, buf: &[u8]) -> IoResult<usize> {
            Write::write(&mut { self }, buf)
        }
        fn flush(&self) -> IoResult<()> { Write::flush(&mut { self }) }
    }
    trait SyncStream: 'static + Send + Sync {
        fn read_alloc(&self, out: &mut OutputBuffer) -> IoResult<()> {
            let mut buf = [0u8; 8192];
            let len = self.read(&mut buf)?;
            out.set(&buf[..len]);
            Ok(())
        }
        fn read(&self, buf: &mut [u8])
        -> IoResult<usize>;
        fn write(&self, buf: &[u8])
        -> IoResult<usize>;
        fn flush(&self)
        -> IoResult<()>;
    }
    trait SyncListener: 'static + Send + Sync {
        fn accept(&self)
        -> IoResult<(FileDesc, Box<ToString>, Box<ToString>)>;
    }
    impl SyncListener for TcpListener {
        fn accept(&self)
         -> IoResult<(FileDesc, Box<ToString>, Box<ToString>)> {
            TcpListener::accept(self).map(|(s, peer)|
                                              {
                                                  let local =
                                                      match s.local_addr() {
                                                          Ok(local) =>
                                                          Box::new(local) as
                                                              _,
                                                          Err(_) =>
                                                          Box::new("error") as
                                                              _,
                                                      };
                                                  (FileDesc::stream(s), local,
                                                   Box::new(peer) as _)
                                              })
        }
    }
    enum FileDesc { Stream(Box<SyncStream>), Listener(Box<SyncListener>), }
    impl FileDesc {
        fn stream<S: SyncStream>(s: S) -> FileDesc {
            FileDesc::Stream(Box::new(s))
        }
        fn listener<L: SyncListener>(l: L) -> FileDesc {
            FileDesc::Listener(Box::new(l))
        }
        fn as_stream(&self) -> IoResult<&SyncStream> {
            if let FileDesc::Stream(ref s) = self {
                Ok(&**s)
            } else { Err(IoErrorKind::InvalidInput.into()) }
        }
        fn as_listener(&self) -> IoResult<&SyncListener> {
            if let FileDesc::Listener(ref l) = self {
                Ok(&**l)
            } else { Err(IoErrorKind::InvalidInput.into()) }
        }
    }
    pub(crate) enum EnclaveAbort<T> {
        Exit {
            panic: T,
        },

        /// Secondary threads exiting due to an abort
        Secondary,
        IndefiniteWait,
        InvalidUsercall(u64),
        MainReturned,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl <T: ::std::fmt::Debug> ::std::fmt::Debug for EnclaveAbort<T> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match (&*self,) {
                (&EnclaveAbort::Exit { panic: ref __self_0 },) => {
                    let mut debug_trait_builder = f.debug_struct("Exit");
                    let _ = debug_trait_builder.field("panic", &&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&EnclaveAbort::Secondary,) => {
                    let mut debug_trait_builder = f.debug_tuple("Secondary");
                    debug_trait_builder.finish()
                }
                (&EnclaveAbort::IndefiniteWait,) => {
                    let mut debug_trait_builder =
                        f.debug_tuple("IndefiniteWait");
                    debug_trait_builder.finish()
                }
                (&EnclaveAbort::InvalidUsercall(ref __self_0),) => {
                    let mut debug_trait_builder =
                        f.debug_tuple("InvalidUsercall");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&EnclaveAbort::MainReturned,) => {
                    let mut debug_trait_builder =
                        f.debug_tuple("MainReturned");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[structural_match]
    #[rustc_copy_clone_marker]
    struct TcsAddress(usize);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::marker::Copy for TcsAddress { }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::clone::Clone for TcsAddress {
        #[inline]
        fn clone(&self) -> TcsAddress {
            { let _: ::std::clone::AssertParamIsClone<usize>; *self }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::fmt::Debug for TcsAddress {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                TcsAddress(ref __self_0_0) => {
                    let mut debug_trait_builder = f.debug_tuple("TcsAddress");
                    let _ = debug_trait_builder.field(&&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::hash::Hash for TcsAddress {
        fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
            match *self {
                TcsAddress(ref __self_0_0) => {
                    ::std::hash::Hash::hash(&(*__self_0_0), state)
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::Eq for TcsAddress {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            { let _: ::std::cmp::AssertParamIsEq<usize>; }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::PartialEq for TcsAddress {
        #[inline]
        fn eq(&self, other: &TcsAddress) -> bool {
            match *other {
                TcsAddress(ref __self_1_0) =>
                match *self {
                    TcsAddress(ref __self_0_0) =>
                    (*__self_0_0) == (*__self_1_0),
                },
            }
        }
        #[inline]
        fn ne(&self, other: &TcsAddress) -> bool {
            match *other {
                TcsAddress(ref __self_1_0) =>
                match *self {
                    TcsAddress(ref __self_0_0) =>
                    (*__self_0_0) != (*__self_1_0),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::Ord for TcsAddress {
        #[inline]
        fn cmp(&self, other: &TcsAddress) -> ::std::cmp::Ordering {
            match *other {
                TcsAddress(ref __self_1_0) =>
                match *self {
                    TcsAddress(ref __self_0_0) =>
                    match ::std::cmp::Ord::cmp(&(*__self_0_0), &(*__self_1_0))
                        {
                        ::std::cmp::Ordering::Equal =>
                        ::std::cmp::Ordering::Equal,
                        cmp => cmp,
                    },
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::PartialOrd for TcsAddress {
        #[inline]
        fn partial_cmp(&self, other: &TcsAddress)
         -> ::std::option::Option<::std::cmp::Ordering> {
            match *other {
                TcsAddress(ref __self_1_0) =>
                match *self {
                    TcsAddress(ref __self_0_0) =>
                    match ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0),
                                                              &(*__self_1_0))
                        {
                        ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                        =>
                        ::std::option::Option::Some(::std::cmp::Ordering::Equal),
                        cmp => cmp,
                    },
                },
            }
        }
        #[inline]
        fn lt(&self, other: &TcsAddress) -> bool {
            match *other {
                TcsAddress(ref __self_1_0) =>
                match *self {
                    TcsAddress(ref __self_0_0) =>
                    ::std::option::Option::unwrap_or(::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0),
                                                                                         &(*__self_1_0)),
                                                     ::std::cmp::Ordering::Greater)
                        == ::std::cmp::Ordering::Less,
                },
            }
        }
        #[inline]
        fn le(&self, other: &TcsAddress) -> bool {
            match *other {
                TcsAddress(ref __self_1_0) =>
                match *self {
                    TcsAddress(ref __self_0_0) =>
                    ::std::option::Option::unwrap_or(::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0),
                                                                                         &(*__self_1_0)),
                                                     ::std::cmp::Ordering::Greater)
                        != ::std::cmp::Ordering::Greater,
                },
            }
        }
        #[inline]
        fn gt(&self, other: &TcsAddress) -> bool {
            match *other {
                TcsAddress(ref __self_1_0) =>
                match *self {
                    TcsAddress(ref __self_0_0) =>
                    ::std::option::Option::unwrap_or(::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0),
                                                                                         &(*__self_1_0)),
                                                     ::std::cmp::Ordering::Less)
                        == ::std::cmp::Ordering::Greater,
                },
            }
        }
        #[inline]
        fn ge(&self, other: &TcsAddress) -> bool {
            match *other {
                TcsAddress(ref __self_1_0) =>
                match *self {
                    TcsAddress(ref __self_0_0) =>
                    ::std::option::Option::unwrap_or(::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0),
                                                                                         &(*__self_1_0)),
                                                     ::std::cmp::Ordering::Less)
                        != ::std::cmp::Ordering::Less,
                },
            }
        }
    }
    impl ErasedTcs {
        fn address(&self) -> TcsAddress {
            TcsAddress(SgxsTcs::address(self) as _)
        }
    }
    impl fmt::Pointer for TcsAddress {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            (self.0 as *const u8).fmt(f)
        }
    }
    struct StoppedTcs {
        tcs: ErasedTcs,
        event_queue: Receiver<u8>,
    }
    struct RunningTcs {
        enclave: Arc<EnclaveState>,
        pending_event_set: u8,
        pending_events: VecDeque<u8>,
        event_queue: Receiver<u8>,
    }
    enum EnclaveKind { Command(Command), Library(Library), }
    struct CommandSync {
        threads: Vec<StoppedTcs>,
        primary_panic_reason: Option<EnclaveAbort<EnclavePanic>>,
        other_reasons: Vec<EnclaveAbort<EnclavePanic>>,
        running_secondary_threads: usize,
    }
    struct Command {
        data: Mutex<CommandSync>,
        wait_secondary_threads: Condvar,
    }
    struct Library {
        threads: Mutex<Receiver<StoppedTcs>>,
        thread_sender: Mutex<Sender<StoppedTcs>>,
    }
    impl EnclaveKind {
        fn as_command(&self) -> Option<&Command> {
            match self { EnclaveKind::Command(c) => Some(c), _ => None, }
        }
        fn as_library(&self) -> Option<&Library> {
            match self { EnclaveKind::Library(l) => Some(l), _ => None, }
        }
    }
    pub(crate) struct EnclaveState {
        kind: EnclaveKind,
        event_queues: FnvHashMap<TcsAddress, Mutex<Sender<u8>>>,
        fds: Mutex<FnvHashMap<Fd, Arc<FileDesc>>>,
        last_fd: AtomicUsize,
        exiting: AtomicBool,
    }
    impl EnclaveState {
        fn event_queue_add_tcs(event_queues:
                                   &mut FnvHashMap<TcsAddress,
                                                   Mutex<Sender<u8>>>,
                               tcs: ErasedTcs) -> StoppedTcs {
            let (send, recv) = channel();
            if event_queues.insert(tcs.address(), Mutex::new(send)).is_some()
               {
                {
                    ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["duplicate TCS address: "],
                                                                              &match (&tcs.address(),)
                                                                                   {
                                                                                   (arg0,)
                                                                                   =>
                                                                                   [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                ::std::fmt::Pointer::fmt)],
                                                                               }),
                                               &("enclave-runner/src/usercalls/mod.rs",
                                                 322u32, 13u32))
                }
            }
            StoppedTcs{tcs, event_queue: recv,}
        }
        fn new(kind: EnclaveKind,
               event_queues: FnvHashMap<TcsAddress, Mutex<Sender<u8>>>)
         -> Arc<Self> {
            let mut fds = FnvHashMap::default();
            fds.insert(FD_STDIN,
                       Arc::new(FileDesc::stream(Shared(io::stdin()))));
            fds.insert(FD_STDOUT,
                       Arc::new(FileDesc::stream(Shared(io::stdout()))));
            fds.insert(FD_STDERR,
                       Arc::new(FileDesc::stream(Shared(io::stderr()))));
            let last_fd =
                AtomicUsize::new(fds.keys().cloned().max().unwrap() as _);
            Arc::new(EnclaveState{kind,
                                  event_queues,
                                  fds: Mutex::new(fds),
                                  last_fd,
                                  exiting: AtomicBool::new(false),})
        }
        pub(crate) async fn main_entry(main: ErasedTcs,
                                       threads: Vec<ErasedTcs>)
         -> StdResult<(), failure::Error> {
            let mut event_queues =
                FnvHashMap::with_capacity_and_hasher(threads.len() + 1,
                                                     Default::default());
            let main = Self::event_queue_add_tcs(&mut event_queues, main);
            let (mut sync_tx, sync_rx) = sync_channel(0);
            unsafe {
                let mut guard = ThreadSyncSender.mutex.lock().unwrap();
                *guard = Some(sync_tx);
            }
            let threads =
                threads.into_iter().map(|thread|
                                            Self::event_queue_add_tcs(&mut event_queues,
                                                                      thread)).collect();
            let kind =
                EnclaveKind::Command(Command{data:
                                                 Mutex::new(CommandSync{threads,
                                                                        primary_panic_reason:
                                                                            None,
                                                                        other_reasons:
                                                                            <[_]>::into_vec(box
                                                                                                []),
                                                                        running_secondary_threads:
                                                                            0,}),
                                             wait_secondary_threads:
                                                 Condvar::new(),});
            let enclave = EnclaveState::new(kind, event_queues);
            let main_result =
                {
                    #[allow(unused_imports)]
                    use ::tokio_async_await::compat::backward::IntoAwaitable
                        as IntoAwaitableBackward;
                    #[allow(unused_imports)]
                    use ::tokio_async_await::compat::forward::IntoAwaitable as
                        IntoAwaitableForward;
                    use ::tokio_async_await::std_await;
                    #[allow(unused_mut)]
                    let mut e =
                        RunningTcs::entry(enclave.clone(), main,
                                          EnclaveEntry::ExecutableMain);
                    let e = e.into_awaitable();
                    {
                        let mut pinned = e;
                        loop  {
                            if let ::std::task::Poll::Ready(x) =
                                   ::std::future::poll_with_tls_waker(unsafe {
                                                                          ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                      }) {
                                break x ;
                            }
                            yield
                        }
                    }
                };
            let main_panicking =
                match main_result {
                    Err(EnclaveAbort::MainReturned) |
                    Err(EnclaveAbort::InvalidUsercall(_)) |
                    Err(EnclaveAbort::Exit { .. }) => true,
                    Err(EnclaveAbort::IndefiniteWait) |
                    Err(EnclaveAbort::Secondary) | Ok(_) => false,
                };
            let cmd = enclave.kind.as_command().unwrap();
            let mut cmddata = cmd.data.lock().unwrap();
            cmddata.threads.clear();
            enclave.abort_all_threads();
            unsafe {
                let mut guard = ThreadSyncSender.mutex.lock().unwrap();
                *guard = None;
            }
            while sync_rx.recv() != Err(RecvError) { }
            let main_result =
                match (main_panicking, cmddata.primary_panic_reason.take()) {
                    (false, Some(reason)) => Err(reason),
                    _ => main_result,
                };
            match main_result {
                Err(EnclaveAbort::Exit { panic }) => Err(panic.into()),
                Err(EnclaveAbort::IndefiniteWait) => {
                    return Err(::failure::err_msg("All enclave threads are waiting indefinitely without possibility of wakeup"))
                }
                Err(EnclaveAbort::InvalidUsercall(n)) => {
                    return Err(::failure::err_msg(::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["The enclave performed an invalid usercall 0x"],
                                                                                                     &match (&n,)
                                                                                                          {
                                                                                                          (arg0,)
                                                                                                          =>
                                                                                                          [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                       ::std::fmt::LowerHex::fmt)],
                                                                                                      }))))
                }
                Err(EnclaveAbort::MainReturned) =>
                return Err(::failure::err_msg("The enclave returned from the main entrypoint in violation of the specification.")),
                Err(EnclaveAbort::Secondary) => {
                    {
                        ::std::rt::begin_panic("internal error: entered unreachable code",
                                               &("enclave-runner/src/usercalls/mod.rs",
                                                 429u32, 45u32))
                    }
                }
                Ok(_) => Ok(()),
            }
        }
        async fn thread_entry(enclave: Arc<Self>, tcs: StoppedTcs)
         -> StdResult<StoppedTcs, EnclaveAbort<EnclavePanic>> {
            {
                #[allow(unused_imports)]
                use ::tokio_async_await::compat::backward::IntoAwaitable as
                    IntoAwaitableBackward;
                #[allow(unused_imports)]
                use ::tokio_async_await::compat::forward::IntoAwaitable as
                    IntoAwaitableForward;
                use ::tokio_async_await::std_await;
                #[allow(unused_mut)]
                let mut e =
                    RunningTcs::entry(enclave.clone(), tcs,
                                      EnclaveEntry::ExecutableNonMain);
                let e = e.into_awaitable();
                {
                    let mut pinned = e;
                    loop  {
                        if let ::std::task::Poll::Ready(x) =
                               ::std::future::poll_with_tls_waker(unsafe {
                                                                      ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                  }) {
                            break x ;
                        }
                        yield
                    }
                }
            }.map(|(tcs, result)|
                      {
                          {
                              match (&(result), &((0, 0))) {
                                  (left_val, right_val) => {
                                      if !(*left_val == *right_val) {
                                          {
                                              ::std::rt::begin_panic_fmt(&::std::fmt::Arguments::new_v1(&["assertion failed: `(left == right)`\n  left: `",
                                                                                                          "`,\n right: `",
                                                                                                          "`: "],
                                                                                                        &match (&&*left_val,
                                                                                                                &&*right_val,
                                                                                                                &::std::fmt::Arguments::new_v1(&["Expected enclave thread entrypoint to return zero"],
                                                                                                                                               &match ()
                                                                                                                                                    {
                                                                                                                                                    ()
                                                                                                                                                    =>
                                                                                                                                                    [],
                                                                                                                                                }))
                                                                                                             {
                                                                                                             (arg0,
                                                                                                              arg1,
                                                                                                              arg2)
                                                                                                             =>
                                                                                                             [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                          ::std::fmt::Debug::fmt),
                                                                                                              ::std::fmt::ArgumentV1::new(arg1,
                                                                                                                                          ::std::fmt::Debug::fmt),
                                                                                                              ::std::fmt::ArgumentV1::new(arg2,
                                                                                                                                          ::std::fmt::Display::fmt)],
                                                                                                         }),
                                                                         &("enclave-runner/src/usercalls/mod.rs",
                                                                           437u32,
                                                                           17u32))
                                          }
                                      }
                                  }
                              }
                          };
                          tcs
                      })
        }
        pub(crate) fn library(threads: Vec<ErasedTcs>) -> Arc<Self> {
            let mut event_queues =
                FnvHashMap::with_capacity_and_hasher(threads.len(),
                                                     Default::default());
            let (send, recv) = channel();
            for thread in threads {
                send.send(Self::event_queue_add_tcs(&mut event_queues,
                                                    thread)).unwrap();
            }
            let kind =
                EnclaveKind::Library(Library{threads: Mutex::new(recv),
                                             thread_sender:
                                                 Mutex::new(send),});
            EnclaveState::new(kind, event_queues)
        }
        pub(crate) async fn library_entry(enclave: Arc<Self>, p1: u64,
                                          p2: u64, p3: u64, p4: u64, p5: u64)
         -> StdResult<(u64, u64), failure::Error> {
            let thread =
                enclave.kind.as_library().unwrap().threads.lock().unwrap().recv().unwrap();
            match {
                      #[allow(unused_imports)]
                      use ::tokio_async_await::compat::backward::IntoAwaitable
                          as IntoAwaitableBackward;
                      #[allow(unused_imports)]
                      use ::tokio_async_await::compat::forward::IntoAwaitable
                          as IntoAwaitableForward;
                      use ::tokio_async_await::std_await;
                      #[allow(unused_mut)]
                      let mut e =
                          RunningTcs::entry(enclave.clone(), thread,
                                            EnclaveEntry::Library{p1,
                                                                  p2,
                                                                  p3,
                                                                  p4,
                                                                  p5,});
                      let e = e.into_awaitable();
                      {
                          let mut pinned = e;
                          loop  {
                              if let ::std::task::Poll::Ready(x) =
                                     ::std::future::poll_with_tls_waker(unsafe
                                                                        {
                                                                            ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                        }) {
                                  break x ;
                              }
                              yield
                          }
                      }
                  } {
                Err(EnclaveAbort::Exit { panic }) => Err(panic.into()),
                Err(EnclaveAbort::IndefiniteWait) => {
                    return Err(::failure::err_msg("This thread is waiting indefinitely without possibility of wakeup"))
                }
                Err(EnclaveAbort::InvalidUsercall(n)) => {
                    return Err(::failure::err_msg(::alloc::fmt::format(::std::fmt::Arguments::new_v1(&["The enclave performed an invalid usercall 0x"],
                                                                                                     &match (&n,)
                                                                                                          {
                                                                                                          (arg0,)
                                                                                                          =>
                                                                                                          [::std::fmt::ArgumentV1::new(arg0,
                                                                                                                                       ::std::fmt::LowerHex::fmt)],
                                                                                                      }))))
                }
                Err(EnclaveAbort::Secondary) => {
                    return Err(::failure::err_msg("This thread exited because another thread aborted"))
                }
                Err(EnclaveAbort::MainReturned) => {
                    {
                        ::std::rt::begin_panic("internal error: entered unreachable code",
                                               &("enclave-runner/src/usercalls/mod.rs",
                                                 492u32, 48u32))
                    }
                }
                Ok((tcs, result)) => {
                    enclave.kind.as_library().unwrap().thread_sender.lock().unwrap().send(tcs).unwrap();
                    Ok(result)
                }
            }
        }
        fn abort_all_threads(&self) {
            self.exiting.store(true, Ordering::SeqCst);
            for queue in self.event_queues.values() {
                let _ = queue.lock().unwrap().send(EV_ABORT as _);
            }
        }
    }
    #[structural_match]
    enum EnclaveEntry {
        ExecutableMain,
        ExecutableNonMain,
        Library {
            p1: u64,
            p2: u64,
            p3: u64,
            p4: u64,
            p5: u64,
        },
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::PartialEq for EnclaveEntry {
        #[inline]
        fn eq(&self, other: &EnclaveEntry) -> bool {
            {
                let __self_vi =
                    unsafe { ::std::intrinsics::discriminant_value(&*self) }
                        as isize;
                let __arg_1_vi =
                    unsafe { ::std::intrinsics::discriminant_value(&*other) }
                        as isize;
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (&EnclaveEntry::Library {
                         p1: ref __self_0,
                         p2: ref __self_1,
                         p3: ref __self_2,
                         p4: ref __self_3,
                         p5: ref __self_4 }, &EnclaveEntry::Library {
                         p1: ref __arg_1_0,
                         p2: ref __arg_1_1,
                         p3: ref __arg_1_2,
                         p4: ref __arg_1_3,
                         p5: ref __arg_1_4 }) =>
                        (*__self_0) == (*__arg_1_0) &&
                            (*__self_1) == (*__arg_1_1) &&
                            (*__self_2) == (*__arg_1_2) &&
                            (*__self_3) == (*__arg_1_3) &&
                            (*__self_4) == (*__arg_1_4),
                        _ => true,
                    }
                } else { false }
            }
        }
        #[inline]
        fn ne(&self, other: &EnclaveEntry) -> bool {
            {
                let __self_vi =
                    unsafe { ::std::intrinsics::discriminant_value(&*self) }
                        as isize;
                let __arg_1_vi =
                    unsafe { ::std::intrinsics::discriminant_value(&*other) }
                        as isize;
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (&EnclaveEntry::Library {
                         p1: ref __self_0,
                         p2: ref __self_1,
                         p3: ref __self_2,
                         p4: ref __self_3,
                         p5: ref __self_4 }, &EnclaveEntry::Library {
                         p1: ref __arg_1_0,
                         p2: ref __arg_1_1,
                         p3: ref __arg_1_2,
                         p4: ref __arg_1_3,
                         p5: ref __arg_1_4 }) =>
                        (*__self_0) != (*__arg_1_0) ||
                            (*__self_1) != (*__arg_1_1) ||
                            (*__self_2) != (*__arg_1_2) ||
                            (*__self_3) != (*__arg_1_3) ||
                            (*__self_4) != (*__arg_1_4),
                        _ => false,
                    }
                } else { true }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::std::cmp::Eq for EnclaveEntry {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::std::cmp::AssertParamIsEq<u64>;
                let _: ::std::cmp::AssertParamIsEq<u64>;
                let _: ::std::cmp::AssertParamIsEq<u64>;
                let _: ::std::cmp::AssertParamIsEq<u64>;
                let _: ::std::cmp::AssertParamIsEq<u64>;
            }
        }
    }
    #[repr(C)]
    #[allow(unused)]
    enum Greg {
        R8 = 0,
        R9,
        R10,
        R11,
        R12,
        R13,
        R14,
        R15,
        RDI,
        RSI,
        RBP,
        RBX,
        RDX,
        RAX,
        RCX,
        RSP,
        RIP,
        EFL,
        CSGSFS,
        ERR,
        TRAPNO,
        OLDMASK,
        CR2,
    }
    extern "C" fn handle_trap(_signo: c_int, _info: *mut siginfo_t,
                              context: *mut c_void) {
        unsafe {
            let context = &mut *(context as *mut ucontext_t);
            let rip = &mut context.uc_mcontext.gregs[Greg::RIP as usize];
            let inst: *const u8 = *rip as _;
            if *inst == 204 { *rip += 1; }
        }
        return;
    }
    fn trap_attached_debugger(tcs: usize) {
        let _g = DEBUGGER_TOGGLE_SYNC.lock().unwrap();
        let hdl = self::signal::SigHandler::SigAction(handle_trap);
        let sig_action =
            signal::SigAction::new(hdl, signal::SaFlags::empty(),
                                   signal::SigSet::empty());
        unsafe {
            let old =
                signal::sigaction(signal::SIGTRAP, &sig_action).unwrap();
            asm!("int3":  : "{rbx}"(tcs) :  : "volatile");
            signal::sigaction(signal::SIGTRAP, &old).unwrap();
        }
    }
    #[allow(unused_variables)]
    impl RunningTcs {
        async fn entry(enclave: Arc<EnclaveState>, tcs: StoppedTcs,
                       mode: EnclaveEntry)
         -> StdResult<(StoppedTcs, (u64, u64)), EnclaveAbort<EnclavePanic>> {
            let buf = RefCell::new([0u8; 1024]);
            let mut state =
                RunningTcs{enclave,
                           event_queue: tcs.event_queue,
                           pending_event_set: 0,
                           pending_events: Default::default(),};
            let ret =
                {
                    let on_usercall =
                        async move
                            |state: &mut RunningTcs, p1, p2, p3, p4, p5|
                            {
                                dispatch(&mut Handler(&mut *state), p1, p2,
                                         p3, p4, p5)
                            };
                    let (p1, p2, p3, p4, p5) =
                        match mode {
                            EnclaveEntry::Library { p1, p2, p3, p4, p5 } =>
                            (p1, p2, p3, p4, p5),
                            _ => (0, 0, 0, 0, 0),
                        };
                    {
                        #[allow(unused_imports)]
                        use ::tokio_async_await::compat::backward::IntoAwaitable
                            as IntoAwaitableBackward;
                        #[allow(unused_imports)]
                        use ::tokio_async_await::compat::forward::IntoAwaitable
                            as IntoAwaitableForward;
                        use ::tokio_async_await::std_await;
                        #[allow(unused_mut)]
                        let mut e =
                            tcs::enter(tcs.tcs, state, on_usercall, p1, p2,
                                       p3, p4, p5);
                        let e = e.into_awaitable();
                        {
                            let mut pinned = e;
                            loop  {
                                if let ::std::task::Poll::Ready(x) =
                                       ::std::future::poll_with_tls_waker(unsafe
                                                                          {
                                                                              ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                          }) {
                                    break x ;
                                }
                                yield
                            }
                        }
                    }
                };
            let tcs_new;
            let state_new;
            let result;
            match ret {
                Ok((tcs, state, r)) => {
                    tcs_new = tcs;
                    state_new = state;
                    result = r
                }
                Err(_) => {
                    {
                        ::std::rt::begin_panic("internal error: entered unreachable code",
                                               &("enclave-runner/src/usercalls/mod.rs",
                                                 622u32, 23u32))
                    }
                }
            }
            let tcs =
                StoppedTcs{tcs: tcs_new, event_queue: state_new.event_queue,};
            match result {
                Err(EnclaveAbort::Exit { panic: true }) => {
                    trap_attached_debugger(tcs.tcs.address().0 as _);
                    Err(EnclaveAbort::Exit{panic:
                                               EnclavePanic::from(buf.into_inner()),})
                }
                Err(EnclaveAbort::Exit { panic: false }) => Ok((tcs, (0, 0))),
                Err(EnclaveAbort::IndefiniteWait) =>
                Err(EnclaveAbort::IndefiniteWait),
                Err(EnclaveAbort::InvalidUsercall(n)) =>
                Err(EnclaveAbort::InvalidUsercall(n)),
                Err(EnclaveAbort::MainReturned) =>
                Err(EnclaveAbort::MainReturned),
                Err(EnclaveAbort::Secondary) => Err(EnclaveAbort::Secondary),
                Ok(_) if mode == EnclaveEntry::ExecutableMain =>
                Err(EnclaveAbort::MainReturned),
                Ok(result) => Ok((tcs, result)),
            }
        }
        fn lookup_fd(&self, fd: Fd) -> IoResult<Arc<FileDesc>> {
            match self.enclave.fds.lock().unwrap().get(&fd) {
                Some(stream) => Ok(stream.clone()),
                None => Err(IoErrorKind::BrokenPipe.into()),
            }
        }
        fn alloc_fd(&self, stream: FileDesc) -> Fd {
            let fd =
                (self.enclave.last_fd.fetch_add(1,
                                                Ordering::Relaxed).checked_add(1).expect("FD overflow"))
                    as Fd;
            let prev =
                self.enclave.fds.lock().unwrap().insert(fd, Arc::new(stream));
            if true {
                if !prev.is_none() {
                    {
                        ::std::rt::begin_panic("assertion failed: prev.is_none()",
                                               &("enclave-runner/src/usercalls/mod.rs",
                                                 666u32, 9u32))
                    }
                };
            };
            fd
        }
        #[inline(always)]
        fn is_exiting(&self) -> bool {
            self.enclave.exiting.load(Ordering::SeqCst)
        }
        #[inline(always)]
        fn read(&self, fd: Fd, buf: &mut [u8]) -> IoResult<usize> {
            self.lookup_fd(fd)?.as_stream()?.read(buf)
        }
        #[inline(always)]
        fn read_alloc(&self, fd: Fd, buf: &mut OutputBuffer) -> IoResult<()> {
            self.lookup_fd(fd)?.as_stream()?.read_alloc(buf)
        }
        #[inline(always)]
        fn write(&self, fd: Fd, buf: &[u8]) -> IoResult<usize> {
            self.lookup_fd(fd)?.as_stream()?.write(buf)
        }
        #[inline(always)]
        fn flush(&self, fd: Fd) -> IoResult<()> {
            self.lookup_fd(fd)?.as_stream()?.flush()
        }
        #[inline(always)]
        fn close(&self, fd: Fd) {
            self.enclave.fds.lock().unwrap().remove(&fd);
        }
        #[inline(always)]
        fn bind_stream(&self, addr: &[u8],
                       local_addr: Option<&mut OutputBuffer>)
         -> IoResult<Fd> {
            let addr =
                str::from_utf8(addr).map_err(|_|
                                                 IoErrorKind::ConnectionRefused)?;
            let socket = TcpListener::bind(addr)?;
            if let Some(local_addr) = local_addr {
                local_addr.set(socket.local_addr()?.to_string().into_bytes())
            }
            Ok(self.alloc_fd(FileDesc::listener(socket)))
        }
        #[inline(always)]
        fn accept_stream(&self, fd: Fd, local_addr: Option<&mut OutputBuffer>,
                         peer_addr: Option<&mut OutputBuffer>)
         -> IoResult<Fd> {
            let (stream, local, peer) =
                self.lookup_fd(fd)?.as_listener()?.accept()?;
            if let Some(local_addr) = local_addr {
                local_addr.set(local.to_string().into_bytes())
            }
            if let Some(peer_addr) = peer_addr {
                peer_addr.set(peer.to_string().into_bytes())
            }
            Ok(self.alloc_fd(stream))
        }
        #[inline(always)]
        fn connect_stream(&self, addr: &[u8],
                          local_addr: Option<&mut OutputBuffer>,
                          peer_addr: Option<&mut OutputBuffer>)
         -> IoResult<Fd> {
            let addr =
                str::from_utf8(addr).map_err(|_|
                                                 IoErrorKind::ConnectionRefused)?;
            let stream = TcpStream::connect(addr)?;
            if let Some(local_addr) = local_addr {
                match stream.local_addr() {
                    Ok(local) =>
                    local_addr.set(local.to_string().into_bytes()),
                    Err(_) => local_addr.set(&b"error"[..]),
                }
            }
            if let Some(peer_addr) = peer_addr {
                match stream.peer_addr() {
                    Ok(peer) => peer_addr.set(peer.to_string().into_bytes()),
                    Err(_) => peer_addr.set(&b"error"[..]),
                }
            }
            Ok(self.alloc_fd(FileDesc::stream(stream)))
        }
        #[inline(always)]
        fn launch_thread(&self) -> IoResult<()> {
            let command =
                self.enclave.kind.as_command().ok_or(IoErrorKind::InvalidInput)?;
            let mut cmddata = command.data.lock().unwrap();
            let new_tcs =
                cmddata.threads.pop().ok_or(IoErrorKind::WouldBlock)?;
            let enclave = self.enclave.clone();
            let result =
                tokio::spawn_async(async move
                                       {
                                           let sync_sender;
                                           {
                                               let mut guard =
                                                   ThreadSyncSender.mutex.clone();
                                               let m1 = guard.lock().unwrap();
                                               sync_sender =
                                                   m1.clone().unwrap().clone();
                                           }
                                           let ret =
                                               {
                                                   #[allow(unused_imports)]
                                                   use ::tokio_async_await::compat::backward::IntoAwaitable
                                                       as
                                                       IntoAwaitableBackward;
                                                   #[allow(unused_imports)]
                                                   use ::tokio_async_await::compat::forward::IntoAwaitable
                                                       as
                                                       IntoAwaitableForward;
                                                   use ::tokio_async_await::std_await;
                                                   #[allow(unused_mut)]
                                                   let mut e =
                                                       EnclaveState::thread_entry(enclave.clone(),
                                                                                  new_tcs);
                                                   let e = e.into_awaitable();
                                                   {
                                                       let mut pinned = e;
                                                       loop  {
                                                           if let ::std::task::Poll::Ready(x)
                                                                  =
                                                                  ::std::future::poll_with_tls_waker(unsafe
                                                                                                     {
                                                                                                         ::std::pin::Pin::new_unchecked(&mut pinned)
                                                                                                     })
                                                                  {
                                                               break x ;
                                                           }
                                                           yield
                                                       }
                                                   }
                                               };
                                           sync_sender.send(0);
                                           return ();
                                       });
            return Ok(());
        }
        #[inline(always)]
        fn exit(&mut self, panic: bool) -> EnclaveAbort<bool> {
            self.enclave.abort_all_threads();
            EnclaveAbort::Exit{panic,}
        }
        fn check_event_set(set: u64) -> IoResult<u8> {
            const EV_ALL: u64 =
                EV_USERCALLQ_NOT_FULL | EV_RETURNQ_NOT_EMPTY | EV_UNPARK;
            if (set & !EV_ALL) != 0 {
                return Err(IoErrorKind::InvalidInput.into());
            }
            if !((EV_ALL | EV_ABORT) <= u8::max_value().into()) {
                {
                    ::std::rt::begin_panic("assertion failed: (EV_ALL | EV_ABORT) <= u8::max_value().into()",
                                           &("enclave-runner/src/usercalls/mod.rs",
                                             823u32, 9u32))
                }
            };
            if !((EV_ALL & EV_ABORT) == 0) {
                {
                    ::std::rt::begin_panic("assertion failed: (EV_ALL & EV_ABORT) == 0",
                                           &("enclave-runner/src/usercalls/mod.rs",
                                             824u32, 9u32))
                }
            };
            Ok(set as u8)
        }
        #[inline(always)]
        fn wait(&mut self, event_mask: u64, timeout: u64) -> IoResult<u64> {
            let wait =
                match timeout {
                    WAIT_NO => false,
                    WAIT_INDEFINITE => true,
                    _ => return Err(IoErrorKind::InvalidInput.into()),
                };
            let event_mask = Self::check_event_set(event_mask)?;
            let mut ret = None;
            if (self.pending_event_set & event_mask) != 0 {
                if let Some(pos) =
                       self.pending_events.iter().position(|ev|
                                                               (ev &
                                                                    event_mask)
                                                                   != 0) {
                    ret = self.pending_events.remove(pos);
                    self.pending_event_set =
                        self.pending_events.iter().fold(0, |m, ev| m | ev);
                }
            }
            if ret.is_none() {
                loop  {
                    let ev =
                        if wait {
                            self.event_queue.recv()
                        } else {
                            match self.event_queue.try_recv() {
                                Ok(ev) => Ok(ev),
                                Err(mpsc::TryRecvError::Disconnected) =>
                                Err(mpsc::RecvError),
                                Err(mpsc::TryRecvError::Empty) => break ,
                            }
                        }.expect("TCS event queue disconnected");
                    if (ev & (EV_ABORT as u8)) != 0 {
                        return Err(IoErrorKind::Other.into());
                    }
                    if (ev & event_mask) != 0 {
                        ret = Some(ev);
                        break ;
                    } else {
                        self.pending_events.push_back(ev);
                        self.pending_event_set |= ev;
                    }
                }
            }
            if let Some(ret) = ret {
                Ok(ret.into())
            } else { Err(IoErrorKind::WouldBlock.into()) }
        }
        #[inline(always)]
        fn send(&self, event_set: u64, target: Option<Tcs>) -> IoResult<()> {
            let event_set = Self::check_event_set(event_set)?;
            if event_set == 0 {
                return Err(IoErrorKind::InvalidInput.into());
            }
            if let Some(tcs) = target {
                let tcs = TcsAddress(tcs.as_ptr() as _);
                let queue =
                    self.enclave.event_queues.get(&tcs).ok_or(IoErrorKind::InvalidInput)?;
                queue.lock().unwrap().send(event_set).expect("TCS event queue disconnected");
            } else {
                for queue in self.enclave.event_queues.values() {
                    let _ = queue.lock().unwrap().send(event_set);
                }
            }
            Ok(())
        }
        #[inline(always)]
        fn insecure_time(&mut self) -> u64 {
            let time =
                time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap();
            (time.subsec_nanos() as u64) + time.as_secs() * 1000000000
        }
        #[inline(always)]
        fn alloc(&self, size: usize, alignment: usize) -> IoResult<*mut u8> {
            unsafe {
                let layout =
                    Layout::from_size_align(size,
                                            alignment).map_err(|_|
                                                                   IoErrorKind::InvalidInput)?;
                if layout.size() == 0 {
                    return Err(IoErrorKind::InvalidInput.into());
                }
                let ptr = System.alloc(layout);
                if ptr.is_null() {
                    Err(IoErrorKind::Other.into())
                } else { Ok(ptr) }
            }
        }
        #[inline(always)]
        fn free(&self, ptr: *mut u8, size: usize, alignment: usize)
         -> IoResult<()> {
            unsafe {
                let layout =
                    Layout::from_size_align(size,
                                            alignment).map_err(|_|
                                                                   IoErrorKind::InvalidInput)?;
                if size == 0 { return Ok(()); }
                Ok(System.dealloc(ptr, layout))
            }
        }
        #[inline(always)]
        fn async_queues(&self, usercall_queue: &mut FifoDescriptor<Usercall>,
                        return_queue: &mut FifoDescriptor<Return>)
         -> IoResult<()> {
            Err(IoErrorKind::Other.into())
        }
    }
}
pub use command::Command;
pub use library::Library;
pub use loader::{EnclaveBuilder, EnclavePanic};
    Finished dev [unoptimized + debuginfo] target(s) in 0.63s
