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

// 避免与ts中的Record类型冲突，改名为ExerciseRecord
export interface ExerciseRecord {
    id: number;
    exercise_id: number;
    // ISO 8601 字符串
    created_at: string;
    weight: number;
    reps: number | null;
}