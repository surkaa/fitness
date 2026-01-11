export interface Routine {
    id: number;
    name: string;
    description: string | null;
}

export interface Exercise {
    id: number;
    routineId: number;
    name: string;
    targetSets: number;
    targetReps: string;
    note: string | null;
    unit: string;
}

// 避免与ts中的Record类型冲突，改名为ExerciseRecord
export interface ExerciseRecord {
    id: number;
    exerciseId: number;
    // ISO 8601 字符串
    createdAt: string;
    weight: number;
    reps: number | null;
}