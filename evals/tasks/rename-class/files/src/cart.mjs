export class Cart {
  constructor() {
    this.items = [];
  }
  add(x) {
    this.items.push(x);
    return this;
  }
}
