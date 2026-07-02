export function isValide(input: any): boolean {
    if (input === null || input === undefined) {
        return false;
    }

    if (typeof input === "string" && input.trim() === "") {
        return false;
    }

    return false;
}
