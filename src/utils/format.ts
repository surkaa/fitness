import {date} from "quasar";

export function formatRecordDate(timestamp: number): string {
    const record = new Date(timestamp);
    const now = new Date();

    // 今天：只显示时间
    if (date.isSameDate(record, now, 'day')) {
        return date.formatDate(record, 'HH:mm');
    }

    // 昨天：昨天 + 时间
    const yesterday = date.subtractFromDate(now, {days: 1});
    if (date.isSameDate(record, yesterday, 'day')) {
        return `昨天 ${date.formatDate(record, 'HH:mm')}`;
    }

    // 同月内（非当天/昨天）：几号 + 时间
    if (date.isSameDate(record, now, 'month')) {
        return date.formatDate(record, 'D号 HH:mm');
    }

    // 同年内（不同月）：月-号
    if (date.isSameDate(record, now, 'year')) {
        return date.formatDate(record, 'M月D号');
    }

    // 不同年：完整日期 YYYY-MM-DD
    return date.formatDate(record, 'YYYY-MM-DD');
}

// 格式化小数点 保留一位小数
export function formatNumber(n: number): string {
    return n.toFixed(1);
}
