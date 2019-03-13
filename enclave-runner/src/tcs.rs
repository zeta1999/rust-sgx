/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//extern crate futures_await as futures;

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
use futures::task::Poll;
use crate::tcs::CoResult::Yield;


pub(crate) type DebugBuffer = [u8; 1024];

//#[async]

pub(crate) fn enter<T: Tcs+Unpin, F:Unpin, R:Unpin, S: Unpin>(
    tcs: T,
    mut state: S,
    mut on_usercall: F,
    p1: u64,
    p2: u64,
    p3: u64,
    p4: u64,
    p5: u64,
) ->  impl Future<Output=(T, DispatchResult, S)>
where
    F: FnMut(S, u64, u64, u64, u64, u64) -> R,
    R: Future<Output = (DispatchResult,S)>
{
    struct Enclave < T:Tcs,F,R, S>
    {
        on_usercall: F,
        state: Option<EnclaveState<T,R,S>>,
    }
    enum EnclaveState <T:Tcs,R,S>
    {
        CoResult{
            result: CoResult<Usercall<T>, (T,u64,u64)>,
            ouc_state: S
        },
        InUsercall {
            future : R,
            usercall : Usercall<T>
        }
    }
    impl <T:Tcs+Unpin,F: Unpin, R: Unpin, S: Unpin> Future for Enclave<T,F,R,S>
        where
            F: FnMut(S, u64, u64, u64, u64, u64) -> R,
            R: Future<Output = (DispatchResult,S)>
    {
        type Output = (T, DispatchResult, S);
        fn poll(self: std::pin::Pin<&mut Self>, waker: &std::task::Waker) -> Poll<Self::Output> {
            let self_ = self.get_mut();
            loop {
                self_.state = Some(match self_.state.take().unwrap() {
                    EnclaveState::InUsercall { mut future, usercall } => {
                        match std::pin::Pin::new(&mut future).poll(waker) {
                            Poll::Pending => return Poll::Pending,
                            Poll::Ready((a, ouc_state)) => match a {
                                Ok(ret) => { EnclaveState::CoResult {
                                    result: usercall.coreturn(ret),
                                    ouc_state
                                    }
                                },
                                Err(err) => return Poll::Ready((usercall.tcs, Err(err), ouc_state)),
                            }
                        }
                    }
                    EnclaveState::CoResult{result: CoResult::Yield(usercall), ouc_state} => {
                        let (p1, p2, p3, p4, p5) = usercall.parameters();
                        EnclaveState::InUsercall { future: (self_.on_usercall)(ouc_state, p1, p2, p3, p4, p5), usercall }
                    }

                    EnclaveState::CoResult{result: CoResult::Return((tcs, v1, v2)), ouc_state} => return Poll::Ready((tcs, Ok((v1, v2)), ouc_state)),
                });
            }
        }
    }
    Enclave {
        on_usercall,
        state:Some(EnclaveState::CoResult {
            result: coenter(tcs, p1, p2, p3, p4, p5),
            ouc_state: state
        })
    }
}

#[derive(Debug)]
pub enum CoResult<Y, R> {
    Yield(Y),
    Return(R),
}

#[derive(Debug)]
pub struct Usercall<T: Tcs> {
    tcs: T,
    parameters: (u64, u64, u64, u64, u64),
}

pub type ThreadResult<T> = CoResult<Usercall<T>, (T, u64, u64)>;

impl<T: Tcs> Usercall<T> {
    pub fn parameters(&self) -> (u64, u64, u64, u64, u64) {
        self.parameters
    }

    pub fn coreturn(
        self,
        retval: (u64, u64),
        //debug_buf: Option<&RefCell<DebugBuffer>>,
    ) -> ThreadResult<T> {
        coenter(self.tcs, 0, retval.0, retval.1, 0, 0)
//        coenter(self.tcs, 0, retval.0, retval.1, 0, 0, debug_buf)
    }
}

pub(crate) fn coenter<T: Tcs>(
    tcs: T,
    mut p1: u64,
    mut p2: u64,
    mut p3: u64,
    mut p4: u64,
    mut p5: u64,
//    debug_buf: Option<&RefCell<DebugBuffer>>,
) -> ThreadResult<T> {
    let sgx_result: u32;
    let mut _tmp: (u64, u64);

    unsafe {
        let debug_buf :Option<&RefCell<DebugBuffer>> = None;
        let mut uninit_debug_buf: DebugBuffer;
        let debug_buf = debug_buf.map(|r| r.borrow_mut());
        let debug_buf = match debug_buf {
            Some(mut buf) => buf.as_mut_ptr(),
            None => {
                uninit_debug_buf = std::mem::uninitialized();
                uninit_debug_buf.as_mut_ptr()
            }
        };
        asm!("
		lea 1f(%rip),%rcx
1:
		enclu
"		: "={eax}"(sgx_result), "={rbx}"(_tmp.0), "={r10}"(_tmp.1),
              "={rdi}"(p1), "={rsi}"(p2), "={rdx}"(p3), "={r8}"(p4), "={r9}"(p5)
            : "{eax}" (2), "{rbx}"(tcs.address()), "{r10}"(debug_buf),
              "{rdi}"(p1), "{rsi}"(p2), "{rdx}"(p3), "{r8}"(p4), "{r9}"(p5)
            : "rcx", "r11", "memory"
            : "volatile"
        )
    };

    if sgx_result != (Enclu::EExit as u32) {
        panic!("Invalid return value in EAX! eax={}", sgx_result);
    }

    if p1 == 0 {
        CoResult::Return((tcs, p2, p3))
    } else {
        CoResult::Yield(Usercall {
            tcs: tcs,
            parameters: (p1, p2, p3, p4, p5),
        })
    }
}
