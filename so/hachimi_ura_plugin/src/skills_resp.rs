//! [SKILLRESP-1] 从服务器响应明文解析技能数据（不读内存、不碰偏移）
//! 端点：/api/skills_live?hex=<响应 msgpack 明文 hex>

struct P<'a> { b: &'a [u8], i: usize }

impl<'a> P<'a> {
    fn byte(&mut self) -> Option<u8> { let v = *self.b.get(self.i)?; self.i += 1; Some(v) }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.i..self.i + n)?; self.i += n; Some(s)
    }
    fn be(&mut self, n: usize) -> Option<u64> {
        let s = self.take(n)?; let mut v: u64 = 0;
        for x in s { v = (v << 8) | u64::from(*x); }
        Some(v)
    }
    fn skip(&mut self) -> Option<()> {
        let t = self.byte()?;
        match t {
  0x00..=0x7f | 0xe0..=0xff => Some(()),
  0x80..=0x8f => { let n = usize::from(t & 0x0f); for _ in 0..n { self.skip()?; self.skip()?; } Some(()) }
  0x90..=0x9f => { let n = usize::from(t & 0x0f); for _ in 0..n { self.skip()?; } Some(()) }
  0xa0..=0xbf => { let n = usize::from(t & 0x1f); self.take(n)?; Some(()) }
  0xc0 | 0xc2 | 0xc3 => Some(()),
  0xc4 | 0xd9 => { let n = usize::from(self.byte()?); self.take(n)?; Some(()) }
  0xc5 | 0xda => { let n = self.be(2)? as usize; self.take(n)?; Some(()) }
  0xc6 | 0xdb => { let n = self.be(4)? as usize; self.take(n)?; Some(()) }
  0xc7 => { let n = usize::from(self.byte()?); self.take(n + 1)?; Some(()) }
  0xc8 => { let n = self.be(2)? as usize; self.take(n + 1)?; Some(()) }
  0xc9 => { let n = self.be(4)? as usize; self.take(n + 1)?; Some(()) }
  0xca => { self.take(4)?; Some(()) }
  0xcb => { self.take(8)?; Some(()) }
  0xcc | 0xd0 => { self.take(1)?; Some(()) }
  0xcd | 0xd1 => { self.take(2)?; Some(()) }
  0xce | 0xd2 => { self.take(4)?; Some(()) }
  0xcf | 0xd3 => { self.take(8)?; Some(()) }
  0xd4 => { self.take(2)?; Some(()) }
  0xd5 => { self.take(3)?; Some(()) }
  0xd6 => { self.take(5)?; Some(()) }
  0xd7 => { self.take(9)?; Some(()) }
  0xd8 => { self.take(17)?; Some(()) }
  0xdc => { let n = self.be(2)? as usize; for _ in 0..n { self.skip()?; } Some(()) }
  0xdd => { let n = self.be(4)? as usize; for _ in 0..n { self.skip()?; } Some(()) }
  0xde => { let n = self.be(2)? as usize; for _ in 0..n { self.skip()?; self.skip()?; } Some(()) }
  0xdf => { let n = self.be(4)? as usize; for _ in 0..n { self.skip()?; self.skip()?; } Some(()) }
  _ => None,
        }
    }
    fn string(&mut self) -> Option<String> {
        let t = self.byte()?;
        let n = match t {
  0xa0..=0xbf => usize::from(t & 0x1f),
  0xd9 => usize::from(self.byte()?),
  0xda => self.be(2)? as usize,
  _ => return None,
        };
        let s = self.take(n)?;
        Some(String::from_utf8_lossy(s).into_owned())
    }
    fn int(&mut self) -> Option<i64> {
        let t = self.byte()?;
        match t {
  0x00..=0x7f => Some(i64::from(t)),
  0xe0..=0xff => Some(i64::from(t) - 256),
  0xcc => Some(self.be(1)? as i64),
  0xcd => Some(self.be(2)? as i64),
  0xce => Some(self.be(4)? as i64),
  0xcf => Some(self.be(8)? as i64),
  0xd0 => Some(i64::from(self.be(1)? as u8 as i8)),
  0xd1 => Some(i64::from(self.be(2)? as u16 as i16)),
  0xd2 => Some(i64::from(self.be(4)? as u32 as i32)),
  0xd3 => Some(self.be(8)? as i64),
  _ => None,
        }
    }
}

fn arr_of_maps(p: &mut P<'_>, keys: &[&str]) -> String {
    let t = match p.byte() { Some(v) => v, None => return "[]".to_string() };
    let n = match t {
        0x90..=0x9f => usize::from(t & 0x0f),
        0xdc => p.be(2).unwrap_or(0) as usize,
        0xdd => p.be(4).unwrap_or(0) as usize,
        _ => return "[]".to_string(),
    };
    let mut out: Vec<String> = Vec::new();
    for _ in 0..n {
        let mt = match p.byte() { Some(v) => v, None => break };
        let pairs = match mt {
  0x80..=0x8f => usize::from(mt & 0x0f),
  0xde => p.be(2).unwrap_or(0) as usize,
  0xdf => p.be(4).unwrap_or(0) as usize,
  _ => break,
        };
        let mut fields: Vec<String> = Vec::new();
        for _ in 0..pairs {
  let k = match p.string() { Some(k) => k, None => break };
  if keys.contains(&k.as_str()) {
      match p.int() {
          Some(v) => fields.push(format!("\"{}\":{}", k, v)),
          None => { if p.skip().is_none() { break; } }
      }
  } else if p.skip().is_none() { break; }
        }
        out.push(format!("{{{}}}", fields.join(",")));
    }
    format!("[{}]", out.join(","))
}

type Out = (Option<String>, Option<String>, Option<String>, Option<i64>);

fn walk(p: &mut P<'_>, depth: usize, out: &mut Out) -> Option<()> {
    if depth > 14 { return p.skip(); }
    let t = p.byte()?;
    let pairs = match t {
        0x80..=0x8f => usize::from(t & 0x0f),
        0xde => p.be(2)? as usize,
        0xdf => p.be(4)? as usize,
        _ => { p.i -= 1; return p.skip(); }
    };
    for _ in 0..pairs {
        let save = p.i;
        let key = match p.string() { Some(k) => k, None => { p.i = save; p.skip()?; p.skip()?; continue; } };
        match key.as_str() {
  "skill_array" => out.0 = Some(arr_of_maps(p, &["skill_id", "level"])),
  "skill_tips_array" => out.1 = Some(arr_of_maps(p, &["group_id", "rarity", "level"])),
  "skill_upgrade_info_array" => out.2 = Some(arr_of_maps(p, &["condition_id", "total_count", "current_count"])),
  "skill_point" => out.3 = p.int(),
  _ => {
      let s2 = p.i;
      if walk(p, depth + 1, out).is_none() { p.i = s2; p.skip()?; }
  }
        }
    }
    Some(())
}

/// 端点入口
pub fn endpoint(hex: &str) -> String {
    let bytes = super::hex_decode(hex);
    if bytes.is_empty() {
        return r#"{"ok":false,"error":"empty"}"#.to_string();
    }
    let mut p = P { b: &bytes, i: 0 };
    let mut out: Out = (None, None, None, None);
    let ok = walk(&mut p, 0, &mut out).is_some();
    format!(
        r#"{{"ok":{},"in_len":{},"skills":{},"tips":{},"upgrades":{},"skill_point":{}}}"#,
        if ok { "true" } else { "false" },
        bytes.len(),
        out.0.unwrap_or_else(|| "[]".to_string()),
        out.1.unwrap_or_else(|| "[]".to_string()),
        out.2.unwrap_or_else(|| "[]".to_string()),
        out.3.map_or("-1".to_string(), |v| v.to_string()),
    )
}
