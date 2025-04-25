class KeyValuePair<K, V> {
    key: K;
    value: V;

    public constructor(key: K, value: V) {
        this.key = key;
        this.value = value;
    }

    public getKey(): K {
        return this.key;
    }

    public getValue(): V {
        return this.value;
    }
}

export const NoValue = Symbol();

/// An extremely simplistic key-value map.
export class BadMap<K, V> {
    entries: KeyValuePair<K, V>[];

    /// Creates an empty key-value map.
    public constructor() {
        this.entries = [];
    }

    /// Returns the index into `this.entries` for the given key.
    getIndexForKey(key: K): number|null {
        for (let i = 0; i < this.entries.length; i++) {
            if (this.entries[i].getKey() === key) {
                return i;
            }
        }
        return null;
    }

    /// Sets the value for `key` to `value` and returns the previous value.
    ///
    /// Returns the singleton `NoValue` if no value was stored for that key.
    public set(key: K, value: V): V|typeof NoValue {
        const index = this.getIndexForKey(key);
        if (index !== null) {
            const ret = this.entries[index].getValue();
            this.entries[index] = new KeyValuePair(this.entries[index].getKey(), value);
            return ret;
        }
        this.entries.push(new KeyValuePair(key, value));
        return NoValue;
    }

    /// Obtains the value for `key`.
    ///
    /// Returns the singleton `NoValue` if no value was stored for that key.
    public get(key: K): V|typeof NoValue {
        const index = this.getIndexForKey(key);
        if (index !== null) {
            return this.entries[index].getValue();
        }
        return NoValue;
    }

    /// Obtains the value for `key` and removes this key-value pair from the map.
    ///
    /// Returns the singleton `NoValue` if no value was stored for that key.
    public remove(key: K): V|typeof NoValue {
        const index = this.getIndexForKey(key);
        if (index !== null) {
            const removedKvps = this.entries.splice(index, 1);
            return removedKvps[0].getValue();
        }
        return NoValue;
    }

    /// Returns the number of entries in the map.
    public length(): number {
        return this.entries.length;
    }

    /// Returns an array containing all the keys in the map.
    ///
    /// Note that modifying the keys can lead to unexpected behavior of the map, so, like, don't.
    public keys(): K[] {
        const ret: K[] = [];
        for (let i = 0; i < this.entries.length; i++) {
            ret.push(this.entries[i].getKey());
        }
        return ret;
    }
}
