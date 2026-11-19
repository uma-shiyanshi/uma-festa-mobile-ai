package com.ranko.matsuricapture

import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/**
 * [APP-SKILL-121] 技能分兜底：响应里取 skill_array，再查 MasterDB 的 grade_value 求和。
 *
 * 取法改为「标记定位」而不是递归遍历 msgpack：
 *   在响应明文里找 ASCII "skill_array"（msgpack 里就是 fixstr 0xAB + 文本），
 *   文本结束的下一个字节就是数组值，从这里往后解析。
 *   这样前面任何没见过的字段类型都不会让游标失步（实测递归版在完整报文中会走偏）。
 *
 * 评价值只有游戏 MasterDB 有，SO 提供只读 SQL：
 *   GET 127.0.0.1:18765/mdb/raw?sql=SELECT id,grade_value FROM skill_data WHERE id IN (...)
 */
object SkillScore {
    private const val SO = "127.0.0.1"
    private const val PORT = 18765
    @Volatile private var cachedEval = 0
    @Volatile private var cachedCount = 0
    @Volatile private var lastTry = 0L

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
  val u = URL("http://$SO:$PORT$path")
  c = u.openConnection() as HttpURLConnection
  c.connectTimeout = timeoutMs
  c.readTimeout = timeoutMs
  c.requestMethod = "GET"
  c.inputStream.bufferedReader().use { it.readText() }
        } catch (e: Throwable) {
  null
        } finally {
  try { c?.disconnect() } catch (e: Throwable) { }
        }
    }

    private fun refresh() {
        val meta = http("/api/sniff/metadata?n=40") ?: return
        val skills = latestSkills(meta) ?: return
        if (skills.isEmpty()) return
        val ids = skills.map { it.first }.distinct()
        val sql = "SELECT id, grade_value FROM skill_data WHERE id IN (" + ids.joinToString(",") + ")"
        val raw = http("/mdb/raw?sql=" + java.net.URLEncoder.encode(sql, "UTF-8"), 6000) ?: return
        val grade = gradeMap(raw) ?: return
        var total = 0
        for ((id, lv) in skills) total += (grade[id] ?: 0) * levelMult(lv)
        cachedEval = total
        cachedCount = skills.size
    }

    /** 上游口径：等级 1 → 1.0，之后每级 -0.1，下限 0.5（×10 保持整数运算） */
    private fun levelMult(level: Int): Int = if (level <= 1) 10 else maxOf(5, 11 - level)

    private fun latestSkills(metaJson: String): List<Pair<Int, Int>>? {
        val root = runCatching { JSONObject(metaJson) }.getOrNull() ?: return null
        val entries = root.optJSONArray("entries") ?: return null
        for (i in entries.length() - 1 downTo 0) {
  val e = entries.optJSONObject(i) ?: continue
  if (e.optString("direction") != "response") continue
  val hex = e.optString("body_hex", "")
  if (hex.length < 32) continue
  val b = runCatching { hexToBytes(hex) }.getOrNull() ?: continue
  val hit = runCatching { skillsAt(b) }.getOrNull()
  if (hit != null && hit.isNotEmpty()) return hit
        }
        return null
    }

    private fun hexToBytes(hex: String): ByteArray {
        val n = hex.length / 2
        val out = ByteArray(n)
        for (i in 0 until n) {
  val h = Character.digit(hex[i * 2], 16)
  val l = Character.digit(hex[i * 2 + 1], 16)
  if (h < 0 || l < 0) throw IllegalArgumentException("hex")
  out[i] = ((h shl 4) or l).toByte()
        }
        return out
    }

    private fun indexOf(b: ByteArray, needle: ByteArray): Int {
        outer@ for (i in 0..(b.size - needle.size)) {
  for (j in needle.indices) if (b[i + j] != needle[j]) continue@outer
  return i
        }
        return -1
    }

    /** 标记后解析：数组 → 每元素 map，抽 skill_id / level */
    private fun skillsAt(b: ByteArray): List<Pair<Int, Int>>? {
        val marker = "skill_array".toByteArray(Charsets.US_ASCII)
        val idx = indexOf(b, marker)
        if (idx < 0) return null
        var i = idx + marker.size
        if (i >= b.size) return null
        val out = ArrayList<Pair<Int, Int>>()

        fun u8(): Int = b[i++].toInt() and 0xFF
        fun be(n: Int): Int { var v = 0; repeat(n) { v = (v shl 8) or (b[i++].toInt() and 0xFF) }; return v }
        fun skip() {
  val t = u8()
  when {
      t <= 0x7F || t >= 0xE0 -> Unit
      t in 0x80..0x8F -> repeat(t and 0x0F) { skip(); skip() }
      t in 0x90..0x9F -> repeat(t and 0x0F) { skip() }
      t in 0xA0..0xBF -> i += (t and 0x1F)
      t == 0xC0 || t == 0xC2 || t == 0xC3 -> Unit
      t == 0xC4 || t == 0xD9 -> i += u8()
      t == 0xC5 || t == 0xDA -> i += be(2)
      t == 0xC6 || t == 0xDB -> i += be(4)
      t == 0xCA -> i += 4
      t == 0xCB -> i += 8
      t == 0xCC || t == 0xD0 -> i += 1
      t == 0xCD || t == 0xD1 -> i += 2
      t == 0xCE || t == 0xD2 -> i += 4
      t == 0xCF || t == 0xD3 -> i += 8
      t == 0xDC -> repeat(be(2)) { skip() }
      t == 0xDD -> repeat(be(4)) { skip() }
      t == 0xDE -> repeat(be(2)) { skip(); skip() }
      t == 0xDF -> repeat(be(4)) { skip(); skip() }
      else -> throw IllegalStateException("mp")
  }
        }
        fun str(): String? {
  val t = u8()
  val n = when {
      t in 0xA0..0xBF -> t and 0x1F
      t == 0xD9 -> u8()
      t == 0xDA -> be(2)
      else -> return null
  }
  val s = String(b, i, n, Charsets.UTF_8); i += n; return s
        }
        fun num(): Int {
  val t = u8()
  return when {
      t <= 0x7F -> t
      t >= 0xE0 -> t - 256
      t == 0xCC -> be(1)
      t == 0xCD -> be(2)
      t == 0xCE -> be(4)
      t == 0xCF -> be(8)
      t == 0xD0 -> be(1).toByte().toInt()
      t == 0xD1 -> be(2).toShort().toInt()
      t == 0xD2 -> be(4)
      t == 0xD3 -> be(8)
      else -> 0
  }
        }

        val t = u8()
        val n = when {
  t in 0x90..0x9F -> t and 0x0F
  t == 0xDC -> be(2)
  t == 0xDD -> be(4)
  else -> return null
        }
        repeat(n) {
  val mt = u8()
  val mp = when {
      mt in 0x80..0x8F -> mt and 0x0F
      mt == 0xDE -> be(2)
      mt == 0xDF -> be(4)
      else -> 0
  }
  var id = 0; var lv = 1
  repeat(mp) {
      val k = str()
      when (k) {
          "skill_id" -> id = num()
          "level" -> lv = num()
          else -> skip()
      }
  }
  if (id > 0) out.add(id to lv)
        }
        return out
    }

    private fun gradeMap(raw: String): Map<Int, Int>? {
        val out = HashMap<Int, Int>()
        val root = runCatching { JSONObject(raw) }.getOrNull()
        val rows = root?.optJSONArray("rows") ?: root?.optJSONArray("data")
        if (rows != null) {
  for (i in 0 until rows.length()) {
      val o = rows.optJSONObject(i) ?: continue
      val id = o.optInt("id", o.optInt("skill_id", 0))
      if (id > 0) out[id] = o.optInt("grade_value", 0)
  }
        }
        return if (out.isEmpty()) null else out
    }
}
