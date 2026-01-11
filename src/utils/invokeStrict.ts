import {invoke} from "@tauri-apps/api/core";
import {Ref} from "vue";
import {Exercise, ExerciseRecord, Routine} from "../types.ts";

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
        result: number; // 返回 id
    };
    // 删除轮次
    delete_routine: {
        args: { routineId: number };
        result: null; // Result<(), String> 成功通常返回 null
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
        result: number; // 返回 id
    };
    // 删除动作
    delete_exercise: {
        args: { exerciseId: number };
        result: null;
    };
    // 记录一次最大重量
    add_record: {
        args: {
            exerciseId: number;
            weight: number;
            reps: number | null; // Option<i64>
        };
        result: number; // 返回 id
    };
    // 删除记录
    delete_record: {
        args: { recordId: number };
        result: null;
    };
    // 分页获取记录
    page_records: {
        args: {
            exerciseId: number;
            page: number;
            page_size: number;
        };
        result: ExerciseRecord[];
    };
}

/**
 * 强类型封装 invoke
 * @param cmd 下划线命令名
 * @param args 参数对象，小驼峰
 * @param loading 可选的 loading 状态引用
 */
export function invokeStrict<C extends keyof CommandMap>(
    cmd: C,
    args: CommandMap[C]["args"] = {},
    loading?: Ref<boolean>,
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
        invoke<unknown>(cmd, args)
            .then((res) => {
                resolve(res as CommandMap[C]["result"]);
            })
            .catch((e) => {
                if (typeof e === "string") reject(e);
                else if (e instanceof Error) reject(e.message);
                else reject(String(e));
            })
            .finally(() => {
                if (loading) loading.value = false;
            });
    });
}
