import {invoke} from "@tauri-apps/api/core";
import {Ref} from "vue";

export interface Routine {
    id: number;
    name: string;
    description: string | null;
}

export interface Exercise {
    id: number;
    routine_id: number;
    name: string;
    target_sets: number;
    target_reps: string;
    note: string | null;
    unit: string;
}

export interface Record {
    id: number;
    exercise_id: number;
    // ISO 8601 字符串
    created_at: string;
    weight: number;
    reps: number | null;
}

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
        args: { routine_id: number };
        result: null; // Result<(), String> 成功通常返回 null
    };
    // 获取轮次下的动作
    get_exercises: {
        args: { routine_id: number };
        result: Exercise[];
    };
    // 添加动作
    add_exercise: {
        args: {
            routine_id: number;
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
        args: { exercise_id: number };
        result: null;
    };
    // 记录一次最大重量
    add_record: {
        args: {
            exercise_id: number;
            weight: number;
            reps: number | null; // Option<i64>
        };
        result: number; // 返回 id
    };
    // 删除记录
    delete_record: {
        args: { record_id: number };
        result: null;
    };
    // 分页获取记录
    page_records: {
        args: {
            exercise_id: number;
            page: number;
            page_size: number;
        };
        result: Record[];
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
