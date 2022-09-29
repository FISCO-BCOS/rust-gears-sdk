#[cfg(feature = "bcos2sdk_ffi")]
use libc::{c_char, c_int, c_void};

//ffi方式链接native_tassl_sock_wrap库且映射C API
#[cfg(feature = "bcos2sdk_ffi")]
#[link(name = "native_tassl_sock_wrap")]
extern "C" {
    pub fn ssock_create() -> *const c_void;
    pub fn ssock_release(p_void_ssock: *const c_void);
    pub fn ssock_init(
        p_void_ssock: *const c_void,
        ca_crt_file_: *const c_char,
        sign_crt_file_: *const c_char,
        sign_key_file_: *const c_char,
        en_crt_file_: *const c_char,
        en_key_file: *const c_char,
    );

    pub fn ssock_try_connect(
        p_void_ssock: *const c_void,
        host_: *const c_char,
        port_: c_int,
    ) -> c_int;

    pub fn ssock_finish(p_void_ssock: *const c_void) -> c_int;
    pub fn ssock_set_echo_mode(p_void_ssock: *const c_void, mode: c_int);
    pub fn ssock_send(p_void_ssock: *const c_void, buffer: *const c_char, len: c_int) -> c_int;
    pub fn ssock_recv(p_void_ssock: *const c_void, buffer: *mut c_char, buffersize: c_int)
        -> c_int;

}

/*

    extern "C" {
        EXPORT_API void * C_API ssock_create();
        EXPORT_API void  C_API ssock_release(void * p_void_ssock);


        EXPORT_API int  C_API ssock_init(
                    void * p_void_ssock,
                    const char *ca_crt_file_,
                    const char * sign_crt_file_,
                    const char * sign_key_file_,
                    const char * en_crt_file_,
                    const char * en_key_file_
                    );

         EXPORT_API int C_API ssock_try_connect(
                    void * p_void_ssock,
                    const char *host_,const int port_);

        EXPORT_API int  C_API ssock_finish(void * p_void_ssock);

        EXPORT_API void  C_API ssock_set_echo_mode(void * p_void_ssock,int mode);



        EXPORT_API int  C_API ssock_send(void * p_void_ssock,
                    const char * buffer,const int len);
        EXPORT_API int  C_API ssock_recv(void * p_void_ssock,
                        char *buffer,const int buffersize);
    }

*/
