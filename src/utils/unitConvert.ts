// 单位选项
export const unitOptions = [
    {label: '公斤', value: 'kg'},
    {label: '磅', value: 'lb'},
    {label: '片数', value: 'plate'},
] as const;

// 单位转义
export function formatUnit(unit: string) {
    const found = unitOptions.find(option => option.value === unit);
    return found ? found.label : unit;
}