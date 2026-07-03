export default interface Equatable<T> {
    equals(other?: T): boolean;
}
