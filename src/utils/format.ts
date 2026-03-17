// 将 UTC 时间字符串转换为北京时间 Date 对象
function toBeijingTime(isoString: string): Date {
    const utcDate = new Date(isoString);
    // 北京时间为 UTC+8，直接加上8小时毫秒数
    return new Date(utcDate.getTime() + 8 * 60 * 60 * 1000);
}

// 获取当前北京时间
function currentBeijingTime(): Date {
    const now = new Date();
    // 先转到 UTC 时间戳，再加上8小时偏移
    const utcTimestamp = now.getTime() + now.getTimezoneOffset() * 60 * 1000;
    return new Date(utcTimestamp + 8 * 60 * 60 * 1000);
}

// 格式化记录时间
export function formatRecordDate(isoString: string): string {
    const recordDate = toBeijingTime(isoString);
    const now = currentBeijingTime();

    const isSameDay = (d1: Date, d2: Date) =>
        d1.getFullYear() === d2.getFullYear() &&
        d1.getMonth() === d2.getMonth() &&
        d1.getDate() === d2.getDate();

    const isYesterday = (d: Date, today: Date) => {
        const yesterday = new Date(today);
        yesterday.setDate(today.getDate() - 1);
        return isSameDay(d, yesterday);
    };

    const isSameMonth = (d1: Date, d2: Date) =>
        d1.getFullYear() === d2.getFullYear() &&
        d1.getMonth() === d2.getMonth();

    const isSameYear = (d1: Date, d2: Date) =>
        d1.getFullYear() === d2.getFullYear();

    // 格式化时间 HH:mm
    const formatTime = (date: Date) =>
        `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;

    if (isSameDay(recordDate, now)) {
        // 当天：只显示时间
        return formatTime(recordDate);
    }

    if (isYesterday(recordDate, now)) {
        // 昨天：昨天 + 时间
        return `昨天 ${formatTime(recordDate)}`;
    }

    if (isSameMonth(recordDate, now)) {
        // 当月内（非当天/昨天）：几号 + 时间
        return `${recordDate.getDate()}日 ${formatTime(recordDate)}`;
    }

    if (isSameYear(recordDate, now)) {
        // 同年不同月：月-日
        return `${recordDate.getMonth() + 1}月${recordDate.getDate()}日`;
    }

    // 不同年：完整日期 YYYY-MM-DD
    return `${recordDate.getFullYear()}-${String(recordDate.getMonth() + 1).padStart(2, '0')}-${String(recordDate.getDate()).padStart(2, '0')}`;
}