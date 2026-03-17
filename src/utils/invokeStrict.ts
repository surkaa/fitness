import {invoke} from "@tauri-apps/api/core";
import {Ref} from "vue";
import {Exercise, ExerciseRecord, ExerciseStats, Routine} from "../types.ts";
import {QVueGlobals} from "quasar";

/**
 * CommandMap: 定义每个 Tauri 命令的参数和返回值
 * cmd: 和 Rust 命令一致，下划线小写
 * args: 参数类型，小驼峰
 * result: 成功返回值类型
 */
interface CommandMap {
    // 获取所有轮次
    get_routines: {
        args: {};
        result: Routine[];
    };
    // 创建轮次
    create_routine: {
        args: { name: string; desc: string };
        result: number;
    };
    // 删除轮次
    delete_routine: {
        args: { routineId: number };
        result: void;
    };
    // 更新轮次
    update_routine: {
        args: { routineId: number; name: string; desc: string };
        result: void;
    };
    // 获取轮次下的动作
    get_exercises: {
        args: { routineId: number };
        result: Exercise[];
    };
    // 添加动作
    add_exercise: {
        args: {
            routineId: number;
            name: string;
            sets: number;
            reps: string;
            note: string;
            unit: string;
        };
        result: number;
    };
    // 删除动作
    delete_exercise: {
        args: { exerciseId: number };
        result: void;
    };
    // 更新动作
    update_exercise: {
        args: {
            exerciseId: number;
            name: string;
            sets: number;
            reps: string;
            note: string;
            unit: string;
        };
        result: void;
    };
    // 记录一次最大重量
    add_record: {
        args: {
            exerciseId: number;
            weight: number;
            reps: number | null; // Option<i64>
        };
        result: number;
    };
    // 删除记录
    delete_record: {
        args: { recordId: number };
        result: void;
    };
    // 分页获取记录
    get_all_records: {
        args: {
            exerciseId: number;
        };
        result: ExerciseRecord[];
    };
    // 更新记录
    update_record: {
        args: { recordId: number; weight: number; reps: number | null };
        result: void;
    };
    // 获取单个动作统计
    get_exercise_stats: {
        args: { exerciseId: number };
        result: ExerciseStats;
    };
    // 获取常用次数
    get_common_reps: {
        args: { exerciseId: number };
        result: number[];  // 后端返回 Vec<i64> 对应 number[]
    };
}

/**
 * 强类型封装 invoke
 * @param cmd 下划线命令名
 * @param args 参数对象，小驼峰
 * @param loading 可选的 loading 状态引用
 * @param $q 可选的 Quasar 全局对象引用
 */
export function invokeStrict<C extends keyof CommandMap>(
    cmd: C,
    args: CommandMap[C]["args"] = {},
    loading?: Ref<boolean>,
    $q?: QVueGlobals,
): Promise<CommandMap[C]["result"]> {
    return new Promise((resolve, reject) => {
        if (loading) {
            if (loading.value) {
                reject("可能重复的操作");
                return;
            } else {
                loading.value = true;
            }
        }
        invoke<CommandMap[C]["result"]>(cmd, args)
            .then((res) => {
                resolve(res);
            })
            .catch((e) => {
                if ($q) {
                    console.log(`invoke ${cmd} error:`, e);
                    $q.notify({
                        type: 'negative',
                        message: e instanceof Error ? e.message : String(e),
                    });
                }

                if (typeof e === "string") reject(e);
                else if (e instanceof Error) reject(e.message);
                else reject(String(e));
            })
            .finally(() => {
                if (loading) loading.value = false;
            });
    });
}
