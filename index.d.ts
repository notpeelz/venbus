export declare class Venbus {
    constructor();
    set callbackToggleMute(cb: () => void);
    set callbackToggleDeafen(cb: () => void);
    setMuted(state: boolean): Promise<void>;
    setDeafened(state: boolean): Promise<void>;
    start(): Promise<void>;
}
