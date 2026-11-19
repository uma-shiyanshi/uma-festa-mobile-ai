// so-fix 117/118/119 —— 在 CI runner 上把三处修复打进 so/hachimi_ura_plugin/src/lib.rs
// 幂等：已含 SO-FIX-117 标记则整体跳过。
import fs from 'node:fs';

const SRC = 'so/hachimi_ura_plugin/src/lib.rs';
let s = fs.readFileSync(SRC, 'utf8');

if (s.includes('[SO-FIX-117]')) {
  console.log('already patched (SO-FIX-117 present) — skip');
  process.exit(0);
}

const L = (...lines) => lines.join('\n');
let applied = [];

// ─────────────────────────────────────────────────────────────
// FIX-119 先把所有 get_DataSet 调用点换成 read_scenario_dataset
// （必须在插入 helper 之前做，否则 helper 内部也会被替换成自递归）
// ─────────────────────────────────────────────────────────────
const re = /call_getter_(?:on_instance|ref)\((\w+), (\w+), "get_DataSet"\)/g;
let n = 0;
s = s.replace(re, (m, a, b) => { n++; return 'read_scenario_dataset(' + a + ', ' + b + ')'; });
console.log('get_DataSet call sites replaced:', n);
applied.push('119: ' + n + ' call sites');

// ─────────────────────────────────────────────────────────────
// FIX-119 helper 插入
// ─────────────────────────────────────────────────────────────
const helperAnchor = 'unsafe fn read_succession_factors_json(';
const helper = L(
'/// [SO-FIX-119] 剧本 DataSet 惰性创建。',
'/// 3.29 起 WorkSingleModeScenarioRamen 等类新增 GetOrCreate()，DataSet 不再在构造时生成，',
'/// 未初始化时直接 get_DataSet 返回 null（/debug/ramen_dataset_path 的 dataset_invoke_status',
'/// = "null_result" 就是这条）。顺序：get_DataSet -> GetOrCreate -> 字段 <DataSet>@16。',
'unsafe fn read_scenario_dataset(sc_class: *mut c_void, sc_obj: *const c_void) -> *mut c_void {',
'    if sc_class.is_null() || sc_obj.is_null() {',
'        return ptr::null_mut();',
'    }',
'    let mut ds = call_getter_ref(sc_class, sc_obj, "get_DataSet");',
'    if ds.is_null() {',
'        ds = call_getter_ref(sc_class, sc_obj, "GetOrCreate");',
'        if !ds.is_null() {',
'            ura_log(3, "[SO-FIX-119] DataSet 走 GetOrCreate 拿到");',
'        }',
'    }',
'    if ds.is_null() {',
'        ds = std::ptr::read_unaligned::<usize>((sc_obj as *const u8).add(16) as *const usize)',
'            as *mut c_void;',
'    }',
'    ds',
'}',
''
);
if (!s.includes(helperAnchor)) throw new Error('helper anchor missing');
s = s.replace(helperAnchor, helper + helperAnchor);
applied.push('119: helper inserted');

// ─────────────────────────────────────────────────────────────
// FIX-117 剧本 15 = 祭典杯(Festa)
// ─────────────────────────────────────────────────────────────
const a117 = L(
'        14 => &["get_ScenarioRamen", "get_WorkScenarioRamen", "get_Ramen"], // ★ 拉面杯',
'        _ => &[],',
'    };'
);
const b117 = L(
'        14 => &["get_ScenarioRamen", "get_WorkScenarioRamen", "get_Ramen"], // ★ 拉面杯',
'        // [SO-FIX-117] 剧本15 = 祭典杯(Festa)。旧表漏了这一项，落进 _ => &[]，',
'        // 于是 "Scenario 15 getter: all attempts failed" -> scenario_obj_null -> festa_dataset_null。',
'        // WorkSingleModeCharaData 上确实有 get_ScenarioFesta（字段 ScenarioFesta@1232），实测可直接 invoke。',
'        15 => &["get_ScenarioFesta", "get_WorkScenarioFesta", "get_Festa"], // ★ 祭典杯',
'        _ => &[],',
'    };'
);
if (!s.includes(a117)) throw new Error('117 anchor A missing');
s = s.replace(a117, b117);
applied.push('117: getter table');

const a117b = L(
'        14 => &["get_ScenarioRamen", "get_WorkScenarioRamen", "get_Ramen"],',
'        _ => &[],'
);
const b117b = L(
'        14 => &["get_ScenarioRamen", "get_WorkScenarioRamen", "get_Ramen"],',
'        15 => &["get_ScenarioFesta", "get_WorkScenarioFesta", "get_Festa"], // [SO-FIX-117]',
'        _ => &[],'
);
if (s.includes(a117b)) { s = s.replace(a117b, b117b); applied.push('117: diag table'); }
else console.log('warn: 117 diag anchor missing (diagnostic endpoint only)');

// ─────────────────────────────────────────────────────────────
// FIX-118 SuccessionFactor：getter 不存在，走字段
// ─────────────────────────────────────────────────────────────
const a118 = L(
'    let arr = call_getter_on_instance(chara_class, chara_obj, "get_SuccessionFactor");',
'    if arr.is_null() {',
'        return "[]".to_string();',
'    }',
'    let base = arr as *const u8;'
);
const b118 = L(
'    // [SO-FIX-118] WorkSingleModeCharaData 上没有 get_SuccessionFactor 这个方法',
'    //（日志一直是 "call_getter: \'get_SuccessionFactor\' not found"，所以 succession_factors 恒空）。',
'    // 真实数据在字段 SuccessionFactor(SuccessionFactorInfo)@1096：',
'    //   SuccessionFactorInfo +16 = <InfoList> List<Info>；List 对象 +16 = _items 数组。',
'    // 下游按「数组对象」语义读长度@0x18 / 元素起点@0x20，正好对齐，不动下面的解析。',
'    let mut arr = call_getter_on_instance(chara_class, chara_obj, "get_SuccessionFactor");',
'    if arr.is_null() {',
'        let sf = read_field_value(chara_class, chara_obj, "SuccessionFactor");',
'        if sf.is_null() {',
'            return "[]".to_string();',
'        }',
'        let list = std::ptr::read_unaligned::<usize>((sf as *const u8).add(16) as *const usize)',
'            as *const c_void;',
'        if list.is_null() {',
'            return "[]".to_string();',
'        }',
'        arr = std::ptr::read_unaligned::<usize>((list as *const u8).add(16) as *const usize)',
'            as *mut c_void;',
'        if arr.is_null() {',
'            return "[]".to_string();',
'        }',
'        ura_log(3, "[SO-FIX-118] SuccessionFactor 走字段路径");',
'    }',
'    let base = arr as *const u8;'
);
if (!s.includes(a118)) throw new Error('118 anchor A missing');
s = s.replace(a118, b118);
applied.push('118: field path');

const a118b = L(
'    let fi_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("FactorInfo").as_ptr());',
'    if fi_class.is_null() {',
'        return "[]".to_string();',
'    }',
'    let items = base.add(IL2CPP_LIST_ITEMS_OFF) as *const usize;'
);
const b118b = L(
'    let fi_class = find_class(image, to_cstr("Gallop").as_ptr(), to_cstr("FactorInfo").as_ptr());',
'    if fi_class.is_null() {',
'        // [SO-FIX-118] FactorInfo 在元数据里是空命名空间的嵌套类，find_class(Gallop) 拿不到。',
'        // 这里不提前返回，交给下面「按对象头取真实类」的兜底（call_getter_obscured_int 对',
'        // 空 class 安全返回 -1）。',
'        ura_log(3, "[SO-FIX-118] FactorInfo by name not found, using object header fallback");',
'    }',
'    let items = base.add(IL2CPP_LIST_ITEMS_OFF) as *const usize;'
);
if (!s.includes(a118b)) throw new Error('118 anchor B missing');
s = s.replace(a118b, b118b);
applied.push('118: fi_class fallback');

const a118c = L(
'        let fid = call_getter_obscured_int(fi_class, elem, "get_FactorId");',
'        let flv = call_getter_obscured_int(fi_class, elem, "get_Level");'
);
const b118c = L(
'        let mut fid = call_getter_obscured_int(fi_class, elem, "get_FactorId");',
'        let mut flv = call_getter_obscured_int(fi_class, elem, "get_Level");',
'        // [SO-FIX-118] InfoList 的元素是 SuccessionFactorInfo.Info，不一定是 FactorInfo；',
'        // 拿不到就按对象头取真实类，再不行就当它包着一层 FactorInfo@16。',
'        if fid <= 0 {',
'            let dyn_class = get_class_from_object(elem);',
'            if !dyn_class.is_null() {',
'                fid = call_getter_obscured_int(dyn_class, elem, "get_FactorId");',
'                flv = call_getter_obscured_int(dyn_class, elem, "get_Level");',
'            }',
'        }',
'        if fid <= 0 {',
'            let inner = std::ptr::read_unaligned::<usize>((elem as *const u8).add(16) as *const usize)',
'                as *const c_void;',
'            if !inner.is_null() {',
'                let inner_class = get_class_from_object(inner);',
'                if !inner_class.is_null() {',
'                    fid = call_getter_obscured_int(inner_class, inner, "get_FactorId");',
'                    flv = call_getter_obscured_int(inner_class, inner, "get_Level");',
'                }',
'            }',
'        }'
);
if (!s.includes(a118c)) throw new Error('118 anchor C missing');
s = s.replace(a118c, b118c);
applied.push('118: element fallback');

fs.writeFileSync(SRC, s);
console.log('written', SRC, s.length, 'bytes');
console.log('applied:', applied.join(' | '));