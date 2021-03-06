
/// Implementing the thoughts of Satoshi Nakamoto in https://bitcoin.org/bitcoin.pdf:
pub mod blockchain {
    use sha2::Sha256;
    use sha2::Digest;
    //use std::hash::Hash; // unnecessary, SHA requires the AsRef<[u8]> trait instead of the Hash trait
    use std::sync::Mutex; // (appending Blocks to a Blockchain has to be synchronized)
    use hex::FromHex; // https://stackoverflow.com/questions/52987181/how-can-i-convert-a-hex-string-to-a-u8-slice
    use std::str::FromStr;


    pub type SHAHash = [u8; 32]; // We use SHA-256 and 256/8 = 32 u8's
    pub type Nonce = u128;
    // The Nonce of a Block is the things that's incremented until the Block's hash has the
    // required zero bits (as many as specified by the ZEROS constant).



    /// The number of zeros the hash of a Block has to start with in order for it to be accepted.
    ///
    /// The higher this number, the more difficult it becomes to continue the Blockchain.
    /// The probability of a hash starting with that many zeros is 1 in 2^ZEROS.
    ///
    /// As we use SHA-256 as the hashing algorithm, this number can be anywhere between
    /// 0 (no effort at all) and 255 (essentially impossible) -> therefore stored as a u8
    static ZEROS : u8 = 5;


    /// This is the hash that's stored as the 'previous hash' (prev_hash) for
    /// the very first / initial Block of a Blockchain
    static INITIAL_HASH : SHAHash = [0u8; 32];



    /// A single Block in the Blockchain.
    /// May store all the data of this Block or just parts of it, but at least the root hash of the
    /// Merkle Tree so that the data can be restored at any time, even from an unreliable source.
    #[derive(Clone, Debug)]
    pub struct Block<T : AsRef<[u8]> + Clone> {
        // Block Header:
        prev_hash   : SHAHash, // the hash of the Block that came before this Block
        nonce       : Nonce,   // random data such that the overall hash of this Block starts with ZEROS 0's

        // The actual data of a Block (or just parts of it, but the root hash at minimum)
        // is stored in a Merkle Tree:
        merkle_tree : MerkleTree<T>
    }



    impl<T : AsRef<[u8]> + Clone> Block<T> {

        /// Creates a new Block
        /// - that comes after the block with the given Hash
        /// - with the data from the given MerkleTree
        ///
        /// The Nonce of the new Block has yet to be calculated by calling calculate_nonce()
        /// afterwards ("mining") !!!
        pub fn new(previous_hash : SHAHash, data : MerkleTree<T>) -> Block<T> {
            Block {
                prev_hash : previous_hash,
                nonce : 0, // has yet to be calculated!
                merkle_tree: data
            }
        }

        /// Calculates the Nonce for this Block such that this Block's hash (calculate_hash())
        /// starts with ZEROS 0's.
        /// The higher the ZEROS constant, the more difficult/time-intensive this operation becomes.
        ///
        /// This is essentially the "mining" process.
        pub fn calculate_nonce(&mut self) -> Nonce {
            // Increment this Block's nonce until it's correct, i.e. this Block's hash starts with
            //   ZERO 0's:
            while !self.verify_nonce() {
                self.nonce += 1;
            }
            self.nonce
        }

        /// Returns the hash of this Block.
        /// When calculate_nonce() has been called on this Block beforehand,
        /// the hash will start with ZEROS 0's.
        pub fn calculate_hash(&self) -> SHAHash {
            SHAHash::from(Sha256::new()
                .chain(self.prev_hash)
                .chain(self.nonce.to_be_bytes())
                .chain(self.merkle_tree.get_root_hash()) // Very important to just use the root hash!
                .finalize())
        }

        /// Checks whether the nonce of this Block was chosen correctly, i.e. whether the hash of
        /// this block starts with ZEROS 0's.
        pub fn verify_nonce(&self) -> bool {
            // Check whether self.calculate_hash() starts with ZEROS 0's:

            let hash : SHAHash = self.calculate_hash();

            // Expected:
            static NO_OF_NULL_BYTES : usize = (ZEROS / 8) as usize;
            static NO_OF_0_BITS: usize = (ZEROS % 8) as usize;

            // Actual:
            for byte in hash.iter().take(NO_OF_NULL_BYTES) {
                if *byte != 0u8 {
                    return false;
                }
            }

            if NO_OF_0_BITS == 0 {
                return true;
            } else {
                let last_mixed_byte = hash[NO_OF_NULL_BYTES];
                return format!("{:b}", last_mixed_byte).starts_with(&String::from("0").repeat(NO_OF_0_BITS));
            }



            // Old (I guess probably much less efficient) hexadecimal way of doing it:

            /*
            // Expected:
            static HEXADECIMAL_ZEROS : usize = (ZEROS / 4) as usize; // (each hexadecimal digit = 4 bits)
            static REMAINING_ZERO_BITS : u8 = ZEROS % 4; // (e.g. a hex 1 stills starts with 3 zero bits)

            // Actual:
            let hash_hex : String = format!("{:x?}", self.calculate_hash()); // :x? formats to hex!
            if hash_hex[..HEXADECIMAL_ZEROS] != String::from("0").repeat(HEXADECIMAL_ZEROS) {
                return false; // hash in hexadecimal starts with too few zeros / '0' characters
            }

            // The first character in the hex hash after the run of zeros that might still represent up to three 0 bits:
            let c : char = hash_hex[HEXADECIMAL_ZEROS];
            return match REMAINING_ZERO_BITS {
                0 => true, // no remaining zero bits to check // all begin with zero or more 0 bits!
                1 => c == '0' || c == '1' || c == '2' || c == '3' || c == '4' || c == '5' || c == '6' || c == '7',
                2 => c == '0' || c == '1' || c == '2' || c == '3', // ... begin with two or more 0 bits!
                _ => c == '0' || c == '1', // only HEX0 and HEX1 begin with three or more 0 bits!
            }
             */
            /*
            0 = 0000    5 = 0101    A = 1010    F = 1111
            1 = 0001    6 = 0110    B = 1011
            2 = 0010    7 = 0111    C = 1100
            3 = 0011    8 = 1000    D = 1101
            4 = 0100    9 = 1001    E = 1110
             */
        }

        /// Checks whether the Merkle Tree of this Block is valid.
        pub fn verify_merkle_tree(&self) -> bool {
            self.merkle_tree.verify()
        }

        /// Checks whether this Block is valid (as seen on its own not in its context as part of
        /// a Blockchain).
        /// Combined check of both verify_nonce() and verify_merkle_tree().
        pub fn verify(&self) -> bool {
            self.verify_nonce() && self.verify_merkle_tree()
        }

        /// Removes the storage of all the data in this Block to clean up space/memory.
        /// The data can however be restored later at any point in time using restore_merkle_tree().
        pub fn clear_merkle_tree(&mut self) {
            self.merkle_tree.shrink_to_minimum();
        }

        /// Tries to restore the data of this Block using a MerkleTree coming from an outside
        /// (unreliable) source.
        /// Returns true if the data was restored successfully, i.e. the MerkleTree was correct and
        /// its root hash was equal to the root hash stored in this Block's Header.
        /// Returns false if no data was restored, i.e. the MerkleTree given was somehow invalid.
        pub fn restore_merkle_tree(&mut self, mtree : MerkleTree<T>) -> bool {
            if mtree.verify() && mtree.get_root_hash() == self.merkle_tree.get_root_hash() {
                self.merkle_tree = mtree;
                true
            } else {
                false
            }
        }
    }

    /// A Blockchain chaining Blocks, each of the Blocks storing multiple values of type T.
    #[derive(Debug)]
    pub struct Blockchain<T : AsRef<[u8]> + Clone> {
        blocks : Vec<Block<T>>,
        append_mutex : Mutex<()>
        // Only 1 thread shall be able to append a Block to a Blockchain at a given time
        // (to undefined undefined behaviour). Note that this mutex only locks the appending and
        // obviously not the time-intensive mining process that happens beforehand!
        // (see difference between append_block() and append_data())
    }

    impl<T : AsRef<[u8]> + Clone> Blockchain<T> {

        pub fn new() -> Blockchain<T> {
            Blockchain {
                blocks: Vec::new(),
                append_mutex : Mutex::new(())
            }
        }

        /// Returns the total number of Blocks in this Blockchain.
        pub fn length(&self) -> usize {
            self.blocks.len()
        }

        /// Returns the hash of the last/latest block in this Blockchain
        /// or the INITIAL_HASH when this Blockchain is still empty
        pub fn hash_of_last_block(&self) -> SHAHash {
            match self.blocks.last() {
                Some(last_block) => last_block.calculate_hash(),
                None => INITIAL_HASH,
            }
        }

        /// Verify the correctness of this Blockchain:
        /// - Verifies whether all the Block Hashes are correct AND valid (i.e. start with ZEROS 0's)
        /// - Verifies whether all the Merkle Root Hashes are correct.
        ///
        /// Or to put it differently:
        /// - Checks whether all the 'Previous Hashes' of the Blocks actually are the hash of the
        ///   blocks that comes directly before it.
        /// - Calls .verify() on each of the blocks in this Blockchain (this includes checking all
        ///   of the Merkle Trees for validity!)
        pub fn verify(&self) -> bool {
            let mut previous_hash : SHAHash = INITIAL_HASH;
            for block in self.blocks.iter() {
                if block.prev_hash != previous_hash || !block.verify() { return false; } // inconsistency found!
                previous_hash = block.calculate_hash(); // update the previous_hash
            }
            true // No inconsistencies found in the Blockchain!
        }

        /// Checks whether the given Block has a correct nonce and prev_hash.
        /// If so, appends the given Block to this Blockchain and returns true.
        /// Returns false when the given Block was incorrect and was not appended.
        ///
        /// This function is primarily used for appending Blocks that others publicly announced to
        /// your private copy of the Blockchain.
        /// In order to append your own data, you have to find out the nonce using trial-and-error
        /// first - the append_data() function does that for you.
        pub fn append_block(&mut self, block : Block<T>) -> bool {
            let _guard = self.append_mutex.lock().unwrap(); // <synchronize>
            if block.verify() && block.prev_hash == self.hash_of_last_block() {
                self.blocks.push(block);
                true
            } else {
                false
            }
            // </synchronize> The lock is released automatically here because the MutexGuard goes out of scope!
        }

        /// Takes the data given as a MerkleTree and "mines" a new Block for it, then appends it to
        /// this Blockchain. When the "mining" (nonce calculation) finished but another Block was
        /// appended in the meantime (most likely via an append_block() call), the whole process has
        /// to start over. This means that calling this function can take very long - potentially
        /// forever!
        ///
        /// This function also returns a copy of the "mined" Block so you can announce it to the network!!
        /// (The communication with others on the network is NOT part of this library!!)
        pub fn append_data(&mut self, mtree : MerkleTree<T>) -> Block<T> {
            let mut new_block = Block::new(self.hash_of_last_block(), mtree);
            new_block.calculate_nonce();
            let new_block_clone = new_block.clone();
            self.append_block(new_block);
            new_block_clone

            // ToDo: concurrency!
            //    1) restart calculating a nonce with a/the new prev_hash when either
            //       a) another thread was faster and already appended data in the meantime,
            //       or, much more likely,
            //       b) somebody else publicly announced a new block which you/another thread of you
            //          simply added using a quick append_block() call
        }
    }




    /// In order to be able to reduce the size of the Blockchain / to forget old
    /// no longer necessary to remember data, Blocks store their data in a Merkle Tree.
    ///
    /// The Merkle Tree can be shrunk, either in part or completely (leaving only its root hash),
    /// but deleted data can be restored later at any point in time, even from an unreliable source!
    ///
    /// In a currency Blockchain, the type T would be something representing a transaction.
    ///
    /// For graphics of Merkle Trees, see the Bitcoin paper (https://bitcoin.org/bitcoin.pdf), pp.4+5
    #[derive(Clone, Debug)]
    pub enum MerkleTree<T : AsRef<[u8]> + Clone> {
        Node(SHAHash, Box<MerkleTree<T>>, Box<MerkleTree<T>>), // (binary tree)
        Leaf(SHAHash, Option<T>)
        // - A Leaf with the Option being 'Some' is an actual Leaf of the Merkle Tree storing some data T.
        // - A Leaf with the Option being 'None' could be 2 things:
        // a) an actual Leaf of the full Merkle Tree with just the data T missing OR
        // b) the root of a Merkle Subtree that has been chopped off (when reinserting that,
        //    the Option shall never become 'Some' but the Leaf shall rather be replaced with a Node)
    }

    impl<T : AsRef<[u8]> + Clone> MerkleTree<T> {

        /// Creates a new Merkle Tree with the data from the given Vector.
        /// If your data is in some other data structure, please collect() it into a Vec first.
        ///
        /// Please note that no data may be added later on and that the data also cannot be changed.
        /// Data can however be forgotten to save space and be restored later.
        ///
        /// Important: this function panics when called on an empty Vec!
        pub fn new(vector : Vec<T>) -> MerkleTree<T> where T : Copy {
            match vector.len() {
                0 => panic!("Cannot create a MerkleTree from an empty Vec!"),
                1 => MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[0]).finalize()), Some(vector[0])),
                2 => {
                    let left_leaf = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[0]).finalize()), Some(vector[0]));
                    let right_leaf = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[1]).finalize()), Some(vector[1]));
                    MerkleTree::Node(SHAHash::from(Sha256::new().chain(left_leaf.get_root_hash()).chain(right_leaf.get_root_hash()).finalize()),
                                     Box::new(left_leaf),
                                     Box::new(right_leaf))
                    }
                3 => {
                    let leaf1 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[0]).finalize()), Some(vector[0]));
                    let leaf2 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[1]).finalize()), Some(vector[1]));
                    let leaf3 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[2]).finalize()), Some(vector[2]));
                    let left_subtree = MerkleTree::Node(SHAHash::from(Sha256::new().chain(leaf1.get_root_hash()).chain(leaf2.get_root_hash()).finalize()),
                                                        Box::new(leaf1),
                                                        Box::new(leaf2));
                    MerkleTree::Node(SHAHash::from(Sha256::new().chain(left_subtree.get_root_hash()).chain(leaf3.get_root_hash()).finalize()),
                                     Box::new(left_subtree),
                                     Box::new(leaf3))
                    }
                4 => {
                    let leaf1 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[0]).finalize()), Some(vector[0]));
                    let leaf2 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[1]).finalize()), Some(vector[1]));
                    let leaf3 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[2]).finalize()), Some(vector[2]));
                    let leaf4 = MerkleTree::Leaf(SHAHash::from(Sha256::new().chain(vector[2]).finalize()), Some(vector[2]));
                    let left_subtree = MerkleTree::Node(SHAHash::from(Sha256::new().chain(leaf1.get_root_hash()).chain(leaf2.get_root_hash()).finalize()),
                                                        Box::new(leaf1),
                                                        Box::new(leaf2));
                    let right_subtree = MerkleTree::Node(SHAHash::from(Sha256::new().chain(leaf3.get_root_hash()).chain(leaf4.get_root_hash()).finalize()),
                                                         Box::new(leaf3),
                                                         Box::new(leaf4));
                    MerkleTree::Node(SHAHash::from(Sha256::new().chain(left_subtree.get_root_hash()).chain(right_subtree.get_root_hash()).finalize()),
                                     Box::new(left_subtree),
                                     Box::new(right_subtree))
                    }
                _ => {
                    panic!("ToDo: create a new MerkleTree with more than 4 leaves!"); // ToDo
                    }
            }
        }

        pub fn get_root_hash(&self) -> SHAHash {
            match self {
                &MerkleTree::Node(hash, _, _) => hash,
                &MerkleTree::Leaf(hash, _) => hash
            }
        }

        /// Checks whether this Merkle Tree is valid, i.e. all hashes are correct.
        pub fn verify(&self) -> bool {
            match self {
                MerkleTree::Leaf(_, None) => true, // no data stored at all -> no hashes to match
                MerkleTree::Leaf(hash, Some(t)) => *hash == SHAHash::from(Sha256::new().chain(t).finalize()),
                MerkleTree::Node(hash, left, right) =>
                    *hash == SHAHash::from(Sha256::new().chain(left.get_root_hash()).chain(right.get_root_hash()).finalize())
                    && left.verify() && right.verify() // -> recursion
            }
        }

        // ----- Retrieve data from this MerkleTree: -----

        /// Returns all the data that's currently stored in this Merkle Tree.
        /// This may NOT be all the data when some of it was already forgotten!
        pub fn get_currently_stored_data(&self) -> Vec<T> where T : Copy {
            match self {
                MerkleTree::Leaf(_, None) => vec![],
                MerkleTree::Leaf(_, Some(data)) => vec![*data],
                MerkleTree::Node(_, left, right) => {
                    let mut answer = left.get_currently_stored_data();
                    answer.append(&mut right.get_currently_stored_data());
                    return answer;
                }
            }
        }

        // ----- Exporting & Importing a MerkleTree as XML: -----
        // (the whole point of storing the)

        /// Export this Merkle Tree in an XML format. The XML can be stored somewhere else and
        /// this Merkle Tree shrunk by calling shrink_to_minimum() to save memory.
        pub fn export_xml(&self) -> String where T : ToString {
            match self {
                MerkleTree::Leaf(hash, None) => format!("<leaf hash=\"{:x?}\" />", hash),
                MerkleTree::Leaf(hash, Some(data)) => format!("<leaf hash=\"{:x?}\" data=\"{}\" />", hash, data.to_string()),
                MerkleTree::Node(hash, left, right) =>
                    format!("<node hash=\"{:x?}\"><left>{}</left><right>{}</right></node>", hash, left.export_xml(), right.export_xml())
            }
        }

        /// Please note that the imported MerkleTree is NOT verified!!!
        /// You have to call verify() afterwards - especially when the XML is coming from an
        /// unreliable source!!!
        ///
        /// Returns None when the input XML was invalid.
        pub fn import_xml(xml : String) -> Option<MerkleTree<T>> where T : AsRef<[u8]> + Clone + FromStr {
            return if xml.starts_with("<leaf hash=") {
                let hash: SHAHash = SHAHash::from_hex(xml_helper_parse_attr(&xml, "hash")).ok()?;
                if xml.contains("data=") {
                    let data : T = T::from_str(xml_helper_parse_attr(&xml, "data").as_str()).ok()?;
                    Some(MerkleTree::Leaf(hash, Some(data)))
                } else {
                    Some(MerkleTree::Leaf(hash, None))
                }
            } else if xml.starts_with("<node hash=") {
                let hash: SHAHash = SHAHash::from_hex(xml_helper_parse_attr(&xml, "hash")).ok()?;
                let left: MerkleTree<T> = MerkleTree::import_xml(xml_helper_parse_tag(&xml, "left"))?;
                let right: MerkleTree<T> = MerkleTree::import_xml(xml_helper_parse_tag(&xml, "right"))?;
                Some(MerkleTree::Node(hash, Box::new(left), Box::new(right)))
            } else {
                None
            }
        }

        pub fn contains_hash(&self, search_hash : SHAHash) -> bool {
            match self {
                MerkleTree::Leaf(hash, _) => *hash == search_hash,
                MerkleTree::Node(hash, left, right) =>
                    *hash == search_hash || left.contains_hash(search_hash) || right.contains_hash(search_hash)
            }
        }

        // ----- Grow/Restore: -----

        /// Tries to restore the given element back into this Merkle Tree.
        /// Returns true if the element was restored successfully or if it was already present.
        /// Returns false if the hash of the given element was not found in this Merkle Tree.
        /// If so, you probably have to use restore_subtree() instead.
        pub fn restore_element(&mut self, element : T) -> bool where T : Copy { // ToDo: avoid Copy
            let el_hash : SHAHash = SHAHash::from(Sha256::new().chain(element).finalize()); // the hash of the given element
            match self {
                MerkleTree::Leaf(hash, _) => // Recursion base case:
                    {
                        if *hash == el_hash {
                            *self = MerkleTree::Leaf(*hash, Some(element)); // Restore
                            true
                        } else {
                            false
                        }
                    },
                MerkleTree::Node(_, left, right) => // Recursion:
                    {
                        // Try to restore the element in the left, then in the right subtree:
                        left.restore_element(element) || right.restore_element(element)
                    }
            }
        }

        /// Tries to insert the given subtree into this Merkle Tree.
        /// Returns false when the root hash of the given subtree was not found in this Merkle Tree.
        ///
        /// Please note that this operation can lead to data being added as well as
        /// data being removed!
        ///
        /// Please also note that the given MerkleTree is NOT checked for validity!
        /// That has to be done beforehand if it's coming from an unreliable source!
        pub fn insert_subtree(&mut self, subtree: MerkleTree<T>) -> bool {

            if self.get_root_hash() == subtree.get_root_hash() {
                *self = subtree;
                return true;
            }

            return match self {
                MerkleTree::Leaf(_, _) => false,
                MerkleTree::Node(_, left, right) => {
                    if left.get_root_hash() == subtree.get_root_hash() {
                        *left = Box::new(subtree);
                        true
                    } else if right.get_root_hash() == subtree.get_root_hash() {
                        *right = Box::new(subtree);
                        true
                    } else if left.contains_hash(subtree.get_root_hash()) {
                        left.insert_subtree(subtree)
                    } else if right.contains_hash(subtree.get_root_hash()) {
                        right.insert_subtree(subtree)
                    } else {
                        false
                    }
                }
            }
        }

        /// Tries to restore the given subtree back into this Merkle Tree.
        /// Returns true if the subtree was restored successfully, even if it was already present (or just in parts).
        ///
        /// This function is smarter than insert_subtree(): Instead of simply replacing the current
        /// subtree with the one given as the parameter (which may actually lead to LOSING data
        /// instead of restoring it when the given subtree contains less data than the one that's
        /// currently already stored) this function is lossless - even when the given subtree
        /// contains less data than the one that's currently stored in this Merkle Tree, no data
        /// is lost!
        ///
        /// Returns false if the root hash of the Merkle Tree given was not found in this Merkle Tree.
        /// If so, you probably have to restore a bigger subtree!
        ///
        /// Please note that the given MerkleTree is NOT checked for validity!
        /// That has to be done beforehand if it's coming from an unreliable source!
        pub fn restore_subtree(&mut self, subtree : MerkleTree<T>) -> bool {
            false // ToDo
        }

        // ----- Shrink: -----

        /// Shrinks this Merkle Tree to its minimum size, leaving only its Root Hash.
        pub fn shrink_to_minimum(&mut self) {
            *self = MerkleTree::Leaf(self.get_root_hash(), None);
        }

        /// Looks for an element in this Merkle Tree that's equal to the given element and 'forgets'
        /// it, i.e. deletes it from this Merkle Tree (but keeps the hash).
        /// The element can be restored later by calling restore_element().
        /// However, when larger parts of this Merkle Tree were thrown away, a restore_subtree()
        /// might be necessary.
        ///
        /// Returns false no element equal to the given element was found in this Merkle Tree or
        /// when it already is forgotten (i.e. only its hash still being there).
        ///
        /// When there are multiple element in this Merkle Tree equal to the given one
        /// (which actually shouldn't be the case for most sensible Blockchain applications)
        /// only the leftmost one is forgotten/deleted and true is returned.
        pub fn forget_leaf(&mut self, element : T) -> bool where T : PartialEq + Copy { // ToDo: avoid Copy
            match self {
                MerkleTree::Leaf(_, None) => false,
                MerkleTree::Leaf(hash, Some(data)) =>
                    {
                        if *data == element {
                            *self = MerkleTree::Leaf(*hash, None);
                            true
                        } else {
                            false
                        }
                    },
                MerkleTree::Node(_, left, right) =>
                    left.forget_leaf(element) || right.forget_leaf(element) // || = lazy OR!
            }
        }

        /// Deletes all the data stored in the leaves of this Merkle Tree but leaves the entire
        /// tree structure and all the hashes intact.
        /// This is a less severe operation than shrink_to_minimum, enabling all of the forgotten
        /// elements to be restored individually using restore_element(), in any order.
        ///
        /// This operation makes sense particularly when the datatype T takes up significantly more
        /// storage than a 256-bit hash.
        pub fn forget_all_leaves(&mut self) {
            // Use recursion:
            match self {
                MerkleTree::Leaf(hash, _) =>
                    {
                        *self = MerkleTree::Leaf(*hash, None); // Forget Leaf (data)!
                    },
                MerkleTree::Node(_, left, right) =>
                    {
                        // Forget all leaves of the left and of the right subtree:
                        left.forget_all_leaves();
                        right.forget_all_leaves();
                    },
            }
        }

        /// Deletes the subtree of this Merkle Tree that has the given hash as its root hash.
        /// The root hash itself is kept!
        /// Returns false when this Merkle Tree (currently) does not have a subtree with that hash.
        ///
        /// Calling mtree.forget_subtree(mtree.get_root_hash()) is equivalent to calling
        /// mtree.shrink_to_minimum().
        pub fn forget_subtree(&mut self, hash : SHAHash) -> bool {
            if hash == self.get_root_hash() {
                self.shrink_to_minimum();
                true
            } else {
                match self {
                    MerkleTree::Leaf(_,_) => false, // hash not found!
                    MerkleTree::Node(_, left, right) =>
                        {
                            left.forget_subtree(hash) || right.forget_subtree(hash)
                        }
                }
            }
        }
    }

    fn xml_helper_parse_attr(xml : &String, attr_name : &str) -> String {
        String::from("") // ToDo
    }

    fn xml_helper_parse_tag(xml : &String, tag_name : &str) -> String {
        String::from("") // ToDo
    }

}
