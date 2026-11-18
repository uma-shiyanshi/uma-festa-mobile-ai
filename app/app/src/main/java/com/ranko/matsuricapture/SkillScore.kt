package com.ranko.matsuricapture

import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/**
 * [APP-SKILL-120] 技能分：从「服务器响应」取已持有技能，再查游戏 MasterDB 的评价值。
 *
 * 为什么这么做：SO 的 read_skills() 走 get_SkillDataArray() 反射，实测返回空，
 * 于是 /summary.skills.eval 恒 0，末盘技能分整块丢失（评价档位被压低）。
 * 但技能列表在响应里就有：
 *   data.single_mode_load_common.chara_info.skill_array = [{skill_id, level}]
 * 而评价值表只有游戏 MasterDB 有（skill_data.grade_value），SO 提供只读 SQL：
 *   GET 127.0.0.1:18765/mdb/raw?sql=SELECT id,grade_value FROM skill_data
 *
 * 本对象按需刷新（限频 5 秒），解析失败一律返回 0，不影响主链路。
 */
object SkillScore {
    private const val SO = "127.0.0.1"
    private const val PORT = 18765
    @Volatile private var cachedEval = 0
    @Volatile private var cachedCount = 0
    @Volatile private var lastTry = 0L

    /** 帧里 skills.eval 有值就用它；否则用缓存（并在后台刷新） */
    fun evalOrZero(src: JSONObject?): Int {
        val direct = src?.optJSONObject("skills")?.optInt("eval", 0) ?: 0
        if (direct > 0) return direct
        val now = System.currentTimeMillis()
        if (now - lastTry > 5000L) {
  lastTry = now
  try { Thread { runCatching { refresh() } }.start() } catch (e: Throwable) { }
        }
        return cachedEval
    }

    fun count(): Int = cachedCount

    private fun http(path: String, timeoutMs: Int = 4000): String? {
        var c: HttpURLConnection? = null
        return try {
  c = (URL("http://$SO:$PORT$path").openConnection() as HttpURLConnection)
  c.connectTimeout = timeoutMs
  c.readTimeout = timeoutMs
  c.requestMethod = "GET"
  val body = c.inputStream.bufferedReader().use { it.readText() }
  body
        } catch (e: Throwable) {
  null
        } finally {
  try { c?.disconnect() } catch (e: Throwable) { }
        }
    }

    private fun refresh() {
        val meta = http("/api/sniff/metadata?n=40") ?: return
        val skills = findSkillArray(meta) ?: return
        if (skills.isEmpty()) return
        val ids = skills.map { it.first }.distinct()
        if (ids.isEmpty()) return
        val sql = "SELECT id, grade_value FROM skill_data WHERE id IN (" + ids.joinToString(",") + ")"
        val raw = http("/mdb/raw?sql=" + java.net.URLEncoder.encode(sql, "UTF-8"), 6000) ?: return
        val grade = parseGradeMap(raw) ?: return
        var total = 0
        for ((id, lv) in skills) {
  val g = grade[id] ?: 0
  total += g * levelMult(lv)
        }
        cachedEval = total
        cachedCount = skills.size
    }

    /** 等级倍数：与上游 skill 评价值口径一致（1 级 1.0，之后按 0.1 递减下限 0.5） */
    private fun levelMult(level: Int): Int = if (level <= 1) 10 else maxOf(5, 11 - level)

    /** 从 /api/sniff/metadata 里找最近一条含 skill_array 的响应，解 msgpack 抽 (id, level) */
    private fun findSkillArray(metaJson: String): List<Pair<Int, Int>>? {
        val root = runCatching { JSONObject(metaJson) }.getOrNull() ?: return null
        val entries = root.optJSONArray("entries") ?: return null
        for (i in entries.length() - 1 downTo 0) {
  val e = entries.optJSONObject(i) ?: continue
  if (e.optString("direction") != "response") continue
  val hex = e.optString("body_hex", "")
  if (hex.isEmpty()) continue
  val bytes = runCatching { hexToBytes(hex) }.getOrNull() ?: continue
  val hit = runCatching { scanSkills(bytes) }.getOrNull()
  if (hit != null && hit.isNotEmpty()) return hit
        }
        return null
    }

    private fun hexToBytes(hex: String): ByteArray {
        val n = hex.length / 2
        val out = ByteArray(n)
        for (i in 0 until n) {
  out[i] = ((Character.digit(hex[i * 2], 16) shl 4) or Character.digit(hex[i * 2 + 1], 16)).toByte()
        }
        return out
    }

    // ── 极简 msgpack：只在任意深度找 skill_array ──
    private class P(val b: ByteArray) {
        var i = 0
        fun byte(): Int = b[i++].toInt() and 0xFF
        fun be(n: Int): Long {
  var v = 0L
  repeat(n) { v = (v shl 8) or (b[i++].toLong() and 0xFF) }
  return v
        }
        fun skip() {
  val t = byte()
  when {
      t <= 0x7F || t >= 0xE0 -> Unit
      t in 0x80..0x8F -> repeat(t and 0x0F) { skip(); skip() }
      t in 0x90..0x9F -> repeat(t and 0x0F) { skip() }
      t in 0xA0..0xBF -> i += (t and 0x1F)
      t == 0xC0 || t == 0xC2 || t == 0xC3 -> Unit
      t == 0xC4 || t == 0xD9 -> i += byte()
      t == 0xC5 || t == 0xDA -> i += be(2).toInt()
      t == 0xC6 || t == 0xDB -> i += be(4).toInt()
      t == 0xC7 -> i += byte() + 1
      t == 0xC8 -> i += be(2).toInt() + 1
      t == 0xC9 -> i += be(4).toInt() + 1
      t == 0xCA -> i += 4
      t == 0xCB -> i += 8
      t == 0xCC || t == 0xD0 -> i += 1
      t == 0xCD || t == 0xD1 -> i += 2
      t == 0xCE || t == 0xD2 -> i += 4
      t == 0xCF || t == 0xD3 -> i += 8
      t == 0xD4 -> i += 2
      t == 0xD5 -> i += 3
      t == 0xD6 -> i += 5
      t == 0xD7 -> i += 9
      t == 0xD8 -> i += 17
      t == 0xDC -> repeat(be(2).toInt()) { skip() }
      t == 0xDD -> repeat(be(4).toInt()) { skip() }
      t == 0xDE -> repeat(be(2).toInt()) { skip(); skip() }
      t == 0xDF -> repeat(be(4).toInt()) { skip(); skip() }
      else -> throw IllegalStateException("mp 0x" + Integer.toHexString(t))
  }
        }
        fun str(): String? {
  val t = byte()
  val n = when {
      t in 0xA0..0xBF -> t and 0x1F
      t == 0xD9 -> byte()
      t == 0xDA -> be(2).toInt()
      else -> return null
  }
  val s = String(b, i, n, Charsets.UTF_8)
  i += n
  return s
        }
        fun int(): Int {
  val t = byte()
  return when {
      t <= 0x7F -> t
      t >= 0xE0 -> t - 256
      t == 0xCC -> be(1).toInt()
      t == 0xCD -> be(2).toInt()
      t == 0xCE -> be(4).toInt()
      t == 0xCF -> be(8).toInt()
      t == 0xD0 -> be(1).toByte().toInt()
      t == 0xD1 -> be(2).toShort().toInt()
      t == 0xD2 -> be(4).toInt()
      t == 0xD3 -> be(8).toInt()
      else -> 0
  }
        }
    }

    private fun scanSkills(bytes: ByteArray): List<Pair<Int, Int>>? {
        val p = P(bytes)
        return walk(p, 0)
    }

    private fun walk(p: P, depth: Int): List<Pair<Int, Int>>? {
        if (depth > 14) { p.skip(); return null }
        val t = p.byte()
        val pairs = when {
  t in 0x80..0x8F -> t and 0x0F
  t == 0xDE -> p.be(2).toInt()
  t == 0xDF -> p.be(4).toInt()
  else -> { p.i -= 1; p.skip(); return null }
        }
        for (k in 0 until pairs) {
  val save = p.i
  val key = p.str()
  if (key == null) { p.i = save; p.skip(); p.skip(); continue }
  if (key == "skill_array") return readSkillArray(p)
  val sub = walk(p, depth + 1)
  if (sub != null && sub.isNotEmpty()) return sub
        }
        return null
    }

    private fun readSkillArray(p: P): List<Pair<Int, Int>> {
        val t = p.byte()
        val n = when {
  t in 0x90..0x9F -> t and 0x0F
  t == 0xDC -> p.be(2).toInt()
  t == 0xDD -> p.be(4).toInt()
  else -> return emptyList()
        }
        val out = ArrayList<Pair<Int, Int>>()
        repeat(n) {
  val mt = p.byte()
  val mp = when {
      mt in 0x80..0x8F -> mt and 0x0F
      mt == 0xDE -> p.be(2).toInt()
      mt == 0xDF -> p.be(4).toInt()
      else -> 0
  }
  var id = 0
  var lv = 1
  repeat(mp) {
      val k = p.str()
      when (k) {
          "skill_id" -> id = p.int()
          "level" -> lv = p.int()
          else -> p.skip()
      }
  }
  if (id > 0) out.add(id to lv)
        }
        return out
    }

    /** /mdb/raw 返回 {"rows":[{id:..,grade_value:..}]} 或数组，两种都兼容 */
    private fun parseGradeMap(raw: String): Map<Int, Int>? {
        val out = HashMap<Int, Int>()
        runCatching {
  val root = JSONObject(raw)
  val rows = root.optJSONArray("rows") ?: root.optJSONArray("data") ?: root.optJSONArray("result")
  if (rows != null) {
      for (i in 0 until rows.length()) {
          val o = rows.optJSONObject(i) ?: continue
          val id = o.optInt("id", o.optInt("skill_id", 0))
          val g = o.optInt("grade_value", 0)
          if (id > 0) out[id] = g
      }
  }
        }.onFailure {
  runCatching {
      val arr = JSONArray(raw)
      for (i in 0 until arr.length()) {
          val o = arr.optJSONObject(i) ?: continue
          val id = o.optInt("id", 0)
          val g = o.optInt("grade_value", 0)
          if (id > 0) out[id] = g
      }
  }
        }
        return if (out.isEmpty()) null else out
    }
}
