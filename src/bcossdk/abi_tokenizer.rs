use ethabi::{param_type::ParamType, Error as ABIError, token::{Token}, Uint};
use crate::bcossdk::liteutils::split_param;


/// Ethabi result type
//pub type Result<T> = std::result::Result<T, ABIError>;
/// This trait should be used to parse string values as tokens.
pub trait ABITokenizer {
	/// Tries to parse a string as a token of given type.
	fn tokenize(param: &ParamType, value: &str) -> Result<Token, ABIError> {
		//println!("test");
		match *param {
			ParamType::Address => Self::tokenize_address(value).map(|a| Token::Address(a.into())),
			ParamType::String => Self::tokenize_string(value).map(Token::String),
			ParamType::Bool => Self::tokenize_bool(value).map(Token::Bool),
			ParamType::Bytes => Self::tokenize_bytes(value).map(Token::Bytes),
			ParamType::FixedBytes(len) => Self::tokenize_fixed_bytes(value, len).map(Token::FixedBytes),
			ParamType::Uint(_) => Self::tokenize_uint(value).map(Into::into).map(Token::Uint),
			ParamType::Int(_) => Self::tokenize_int(value).map(Into::into).map(Token::Int),
			ParamType::Array(ref p) => Self::tokenize_array(value, p).map(Token::Array),
			ParamType::FixedArray(ref p, len) => Self::tokenize_fixed_array(value, p, len).map(Token::FixedArray),
			ParamType::Tuple(ref p) => Self::tokenize_struct(value, p).map(Token::Tuple),
		}
	}

	/// Tries to parse a value as a vector of tokens of fixed size.
	fn tokenize_fixed_array(value: &str, param: &ParamType, len: usize) -> Result<Vec<Token>, ABIError> {
		let result = Self::tokenize_array(value, param)?;
		match result.len() == len {
			true => Ok(result),
			false => Err(ABIError::InvalidData),
		}
	}

	/// Tried to parse a struct as a vector of tokens
	fn tokenize_struct(value: &str, param: &Vec<Box<ParamType>>) -> Result<Vec<Token>, ABIError> {
				let inputstr = value.trim_start_matches("(");
        let inputstr = inputstr.trim_end_matches(")");

        let paramstr_array = split_param(inputstr);
		let mut result:Vec<Token> = vec!();
		let mut params = param.iter();
		for sub in paramstr_array
		{
			let token = Self::tokenize(params.next().ok_or(ABIError::InvalidData)?, sub.as_str())?;
			result.push(token);
		}
		Ok(result)
	}

	/// Tries to parse a value as a vector of tokens.
	fn tokenize_array(value: &str, param: &ParamType) -> Result<Vec<Token>, ABIError> {
		let inputstr = value.trim_start_matches("[");
        let inputstr = inputstr.trim_end_matches("]");

        let paramstr_array = split_param(inputstr);
		let mut result:Vec<Token> = vec!();
		for sub in paramstr_array
		{
			let token = Self::tokenize(param, sub.as_str())?;
			result.push(token);
		}
		Ok(result)
	}

	/// Tries to parse a value as an address.
	fn tokenize_address(value: &str) -> Result<[u8; 20],ABIError>;

	/// Tries to parse a value as a string.
	fn tokenize_string(value: &str) -> Result<String,ABIError>;

	/// Tries to parse a value as a bool.
	fn tokenize_bool(value: &str) -> Result<bool,ABIError>;

	/// Tries to parse a value as bytes.
	fn tokenize_bytes(value: &str) -> Result<Vec<u8>,ABIError>;

	/// Tries to parse a value as bytes.
	fn tokenize_fixed_bytes(value: &str, len: usize) -> Result<Vec<u8>,ABIError>;

	/// Tries to parse a value as unsigned integer.
	fn tokenize_uint(value: &str) -> Result<[u8; 32],ABIError>;

	/// Tries to parse a value as signed integer.
	fn tokenize_int(value: &str) -> Result<[u8; 32],ABIError>;
}


pub struct ABIStrictTokenizer;

impl ABITokenizer for ABIStrictTokenizer {
	fn tokenize_address(value: &str) -> Result<[u8; 20],ABIError> {
		let hexres = hex::decode(value);
		let hex = match hexres{
			Ok(h)=>h,
			Err(e)=>return Err(ABIError::InvalidData)
		};
		match hex.len() == 20 {
			false => Err(ABIError::InvalidData),
			true => {
				let mut address = [0u8; 20];
				address.copy_from_slice(&hex);
				Ok(address)
			}
		}
	}

	fn tokenize_string(value: &str) -> Result<String,ABIError> {
		Ok(value.to_owned())
	}

	fn tokenize_bool(value: &str) -> Result<bool,ABIError> {
		match value {
			"true" | "1" => Ok(true),
			"false" | "0" => Ok(false),
			_ => Err(ABIError::InvalidData),
		}
	}

	fn tokenize_bytes(value: &str) -> Result<Vec<u8>,ABIError> {
		let vres= hex::decode(value);
		match vres{
			Ok(v)=>return Ok(v),
			Err(e)=>return Err(ABIError::InvalidData)
		};
	}

	fn tokenize_fixed_bytes(value: &str, len: usize) -> Result<Vec<u8>,ABIError> {
		let hexres = hex::decode(value);
		let hex = match hexres{
			Ok(h)=>h,
			Err(e)=>return Err(ABIError::InvalidData)
		};
		match hex.len() == len {
			true => Ok(hex),
			false => Err(ABIError::InvalidData),
		}
	}

	fn tokenize_uint(value: &str) -> Result<[u8; 32],ABIError> {
		let hexres= hex::decode(value);
		let hex = match hexres{
			Ok(h)=>h,
			Err(e)=>return Err(ABIError::InvalidData)
		};
		match hex.len() == 32 {
			true => {
				let mut uint = [0u8; 32];
				uint.copy_from_slice(&hex);
				Ok(uint)
			}
			false => Err(ABIError::InvalidData),
		}
	}

	fn tokenize_int(value: &str) -> Result<[u8; 32],ABIError> {
		Self::tokenize_uint(value)
	}
}


pub struct ABILenientTokenizer;

impl ABITokenizer for ABILenientTokenizer {
	fn tokenize_address(value: &str) -> Result<[u8; 20],ABIError> {
		ABIStrictTokenizer::tokenize_address(value)
	}

	fn tokenize_string(value: &str) -> Result<String,ABIError> {
		ABIStrictTokenizer::tokenize_string(value)
	}

	fn tokenize_bool(value: &str) -> Result<bool,ABIError> {
		ABIStrictTokenizer::tokenize_bool(value)
	}

	fn tokenize_bytes(value: &str) -> Result<Vec<u8>,ABIError> {
		ABIStrictTokenizer::tokenize_bytes(value)
	}

	fn tokenize_fixed_bytes(value: &str, len: usize) -> Result<Vec<u8>,ABIError> {
		ABIStrictTokenizer::tokenize_fixed_bytes(value, len)
	}

	fn tokenize_uint(value: &str) -> Result<[u8; 32],ABIError> {
		let result = ABIStrictTokenizer::tokenize_uint(value);
		if result.is_ok() {
			return result;
		}

		let uint = Uint::from_dec_str(value)?;
		Ok(uint.into())
	}

	// We don't have a proper signed int 256-bit long type, so here we're cheating. We build a U256
	// out of it and check that it's within the lower/upper bound of a hypothetical I256 type: half
	// the `U256::max_value().
	fn tokenize_int(value: &str) -> Result<[u8; 32],ABIError> {
		let result = ABIStrictTokenizer::tokenize_int(value);
		if result.is_ok() {
			return result;
		}

		let abs = Uint::from_dec_str(value.trim_start_matches('-'))?;
		let max = Uint::max_value() / 2;
		let int = if value.starts_with('-') {
			if abs.is_zero() {
				return Ok(abs.into());
			} else if abs > max + 1 {
				return Err(ABIError::Other("int256 parse error: Underflow".into()));
			}
			!abs + 1 // two's complement
		} else {
			if abs > max {
				return Err(ABIError::Other("int256 parse error: Overflow".into()));
			}
			abs
		};
		Ok(int.into())
	}
}

