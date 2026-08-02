// Package base58 is a thin cgo shim that forwards every call to the Rust port
// (crate b58bridge). It lets the original, unmodified mr-tron/base58 test files
// run against the Rust implementation. This is test scaffolding, not the port.
package base58

/*
#cgo LDFLAGS: -L${SRCDIR}/../target/release -lb58bridge
#include <stdint.h>
#include <stdlib.h>
extern uint8_t* b58_encode(const uint8_t* inp, size_t in_len, const uint8_t* alpha, int fast, size_t* out);
extern uint8_t* b58_decode(const uint8_t* inp, size_t in_len, const uint8_t* alpha, int fast, size_t* out, int32_t* err);
extern int32_t  b58_alphabet_ok(const uint8_t* alpha);
extern void     b58_free(uint8_t* ptr, size_t len);
*/
import "C"

import (
	"errors"
	"unsafe"
)

// Alphabet mirrors the original's opaque type; it just carries the 58 characters.
type Alphabet struct {
	chars [58]byte
}

// NewAlphabet validates through the Rust port and panics on invalid input,
// matching the original's contract.
func NewAlphabet(s string) *Alphabet {
	if len(s) != 58 {
		panic("base58 alphabets must be 58 bytes long")
	}
	a := &Alphabet{}
	copy(a.chars[:], s)
	if C.b58_alphabet_ok((*C.uint8_t)(unsafe.Pointer(&a.chars[0]))) == 0 {
		panic("provided alphabet is not 58 distinct ascii characters")
	}
	return a
}

var BTCAlphabet = NewAlphabet("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz")
var FlickrAlphabet = NewAlphabet("123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ")

func alphaPtr(a *Alphabet) *C.uint8_t {
	if a == nil {
		return nil
	}
	return (*C.uint8_t)(unsafe.Pointer(&a.chars[0]))
}

func bytesPtr(b []byte) *C.uint8_t {
	if len(b) == 0 {
		return nil
	}
	return (*C.uint8_t)(unsafe.Pointer(&b[0]))
}

func encodeImpl(bin []byte, a *Alphabet, fast C.int) string {
	var out C.size_t
	p := C.b58_encode(bytesPtr(bin), C.size_t(len(bin)), alphaPtr(a), fast, &out)
	if p == nil {
		return ""
	}
	defer C.b58_free(p, out)
	return string(C.GoBytes(unsafe.Pointer(p), C.int(out)))
}

func decodeImpl(str string, a *Alphabet, fast C.int) ([]byte, error) {
	data := []byte(str)
	var out C.size_t
	var errc C.int32_t
	p := C.b58_decode(bytesPtr(data), C.size_t(len(data)), alphaPtr(a), fast, &out, &errc)
	if errc != 0 {
		return nil, decodeError(int(errc))
	}
	if p == nil {
		return []byte{}, nil
	}
	defer C.b58_free(p, out)
	return C.GoBytes(unsafe.Pointer(p), C.int(out)), nil
}

func decodeError(code int) error {
	switch code {
	case 1:
		return errors.New("zero length string")
	case 3:
		return errors.New("high-bit set on invalid digit")
	default:
		return errors.New("invalid base58 digit")
	}
}

func Encode(bin []byte) string                       { return encodeImpl(bin, nil, 1) }
func EncodeAlphabet(bin []byte, a *Alphabet) string  { return encodeImpl(bin, a, 1) }
func FastBase58Encoding(bin []byte) string           { return encodeImpl(bin, nil, 1) }
func FastBase58EncodingAlphabet(b []byte, a *Alphabet) string {
	return encodeImpl(b, a, 1)
}
func TrivialBase58Encoding(bin []byte) string { return encodeImpl(bin, nil, 0) }
func TrivialBase58EncodingAlphabet(b []byte, a *Alphabet) string {
	return encodeImpl(b, a, 0)
}

func Decode(str string) ([]byte, error)                      { return decodeImpl(str, nil, 1) }
func DecodeAlphabet(str string, a *Alphabet) ([]byte, error) { return decodeImpl(str, a, 1) }
func FastBase58Decoding(str string) ([]byte, error)          { return decodeImpl(str, nil, 1) }
func FastBase58DecodingAlphabet(str string, a *Alphabet) ([]byte, error) {
	return decodeImpl(str, a, 1)
}
func TrivialBase58Decoding(str string) ([]byte, error) { return decodeImpl(str, nil, 0) }
func TrivialBase58DecodingAlphabet(str string, a *Alphabet) ([]byte, error) {
	return decodeImpl(str, a, 0)
}
