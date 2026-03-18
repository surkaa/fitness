export interface HeaderRightAction {
    icon: string;
    label: string;
    action: () => void;
}


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

export interface ExerciseRecord {
    id: number;
    exerciseId: number;
    // ISO 8601 字符串
    createdAt: string;
    weight: number;
    reps: number | null;
}

export interface ExerciseStats {
    exerciseId: number;
    totalRecords: number;
    maxWeight: number | null;
    lastDate: string | null; // ISO 8601 字符串
}
