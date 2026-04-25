export interface HeaderRightAction {
    icon: string;
    label: string;
    action: () => void;
}

export interface HeaderPrimaryAction {
    icon?: string;
    label: string;
    action: () => void;
}
