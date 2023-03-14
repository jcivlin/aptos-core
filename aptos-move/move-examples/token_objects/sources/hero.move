module token_objects::hero {
    use std::error;
    use std::option::{Self, Option};
    use std::signer;
    use std::string::{Self, String};

    use aptos_framework::object::{Self, ConstructorRef, Object};

    use token_objects::collection;
    use token_objects::token;

    const ENOT_A_HERO: u64 = 1;
    const ENOT_A_WEAPON: u64 = 2;
    const ENOT_A_GEM: u64 = 3;
    const ENOT_CREATOR: u64 = 4;
    const EINVALID_WEAPON_UNEQUIP: u64 = 5;
    const EINVALID_GEM_UNEQUIP: u64 = 6;

    struct OnChainConfig has key {
        collection: String,
        mutability_config: token::MutabilityConfig,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct Hero has key {
        armor: Option<Object<Armor>>,
        gender: String,
        race: String,
        shield: Option<Object<Shield>>,
        weapon: Option<Object<Weapon>>,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct Armor has key {
        defense: u64,
        gem: Option<Object<Gem>>,
        weight: u64,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct Gem has key {
        attack_modifier: u64,
        defense_modifier: u64,
        magic_attribute: String,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct Shield has key {
        defense: u64,
        gem: Option<Object<Gem>>,
        weight: u64,
    }

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    struct Weapon has key {
        attack: u64,
        gem: Option<Object<Gem>>,
        weapon_type: String,
        weight: u64,
    }

    fun init_module(account: &signer) {
        let collection = string::utf8(b"Hero Quest!");
        collection::create_untracked_collection(
            account,
            string::utf8(b"collection description"),
            collection::create_mutability_config(false, false),
            *&collection,
            option::none(),
            string::utf8(b"collection uri"),
        );

        let on_chain_config = OnChainConfig {
            collection: string::utf8(b"Hero Quest!"),
            mutability_config: token::create_mutability_config(true, true, true),
        };
        move_to(account, on_chain_config);
    }

    fun create_token(
        creator: &signer,
        description: String,
        name: String,
        uri: String,
    ): ConstructorRef acquires OnChainConfig {
        let on_chain_config = borrow_global<OnChainConfig>(signer::address_of(creator));
        token::create_token(
            creator,
            *&on_chain_config.collection,
            description,
            *&on_chain_config.mutability_config,
            name,
            option::none(),
            uri,
        )
    }

    // Creation methods

    public fun create_hero(
        creator: &signer,
        description: String,
        gender: String,
        name: String,
        race: String,
        uri: String,
    ): Object<Hero> acquires OnChainConfig {
        let creator_ref = create_token(creator, description, name, uri);
        let token_signer = object::generate_signer(&creator_ref);

        let hero = Hero {
            armor: option::none(),
            gender,
            race,
            shield: option::none(),
            weapon: option::none(),
        };
        move_to(&token_signer, hero);

        object::address_to_object(signer::address_of(&token_signer))
    }

    public fun create_weapon(
        creator: &signer,
        attack: u64,
        description: String,
        name: String,
        uri: String,
        weapon_type: String,
        weight: u64,
    ): Object<Weapon> acquires OnChainConfig {
        let creator_ref = create_token(creator, description, name, uri);
        let token_signer = object::generate_signer(&creator_ref);

        let weapon = Weapon {
            attack,
            gem: option::none(),
            weapon_type,
            weight,
        };
				move_to(&token_signer, weapon);

        object::address_to_object(signer::address_of(&token_signer))
    }

    public fun create_gem(
        creator: &signer,
        attack_modifier: u64,
        defense_modifier: u64,
        description: String,
        magic_attribute: String,
        name: String,
        uri: String,
    ): Object<Gem> acquires OnChainConfig {
        let creator_ref = create_token(creator, description, name, uri);
        let token_signer = object::generate_signer(&creator_ref);

        let gem = Gem {
            attack_modifier,
            defense_modifier,
            magic_attribute,
        };
				move_to(&token_signer, gem);

        object::address_to_object(signer::address_of(&token_signer))
    }

    // Transfer wrappers

    public fun hero_equip_weapon(owner: &signer, hero: Object<Hero>, weapon: Object<Weapon>) acquires Hero {
        let hero_obj = borrow_global_mut<Hero>(object::object_address(&hero));
        option::fill(&mut hero_obj.weapon, weapon);
        object::transfer_to_object(owner, weapon, hero);
    }

    public fun hero_unequip_weapon(owner: &signer, hero: Object<Hero>, weapon: Object<Weapon>) acquires Hero {
        let hero_obj = borrow_global_mut<Hero>(object::object_address(&hero));
        let stored_weapon = option::extract(&mut hero_obj.weapon);
        assert!(stored_weapon == weapon, error::not_found(EINVALID_WEAPON_UNEQUIP));
        object::transfer(owner, weapon, signer::address_of(owner));
    }

    public fun weapon_equip_gem(owner: &signer, weapon: Object<Weapon>, gem: Object<Gem>) acquires Weapon {
        let weapon_obj = borrow_global_mut<Weapon>(object::object_address(&weapon));
        option::fill(&mut weapon_obj.gem, gem);
        object::transfer_to_object(owner, gem, weapon);
    }

    public fun weapon_unequip_gem(owner: &signer, weapon: Object<Weapon>, gem: Object<Gem>) acquires Weapon {
        let weapon_obj = borrow_global_mut<Weapon>(object::object_address(&weapon));
        let stored_gem = option::extract(&mut weapon_obj.gem);
        assert!(stored_gem == gem, error::not_found(EINVALID_GEM_UNEQUIP));
        object::transfer(owner, gem, signer::address_of(owner));
    }

    // Entry functions

    entry fun mint_hero(
        account: &signer,
        description: String,
        gender: String,
        name: String,
        race: String,
        uri: String,
    ) acquires OnChainConfig {
        create_hero(account, description, gender, name, race, uri);
    }

    #[test(account = @0x3)]
    fun test_hero_with_gem_weapon(account: &signer) acquires Hero, OnChainConfig, Weapon {
        init_module(account);

        let hero = create_hero(
            account,
            string::utf8(b"The best hero ever!"),
            string::utf8(b"Male"),
            string::utf8(b"Wukong"),
            string::utf8(b"Monkey God"),
            string::utf8(b""),
        );

        let weapon = create_weapon(
            account,
            32,
            string::utf8(b"A magical staff!"),
            string::utf8(b"Ruyi Jingu Bang"),
            string::utf8(b""),
            string::utf8(b"staff"),
            15,
        );

        let gem = create_gem(
            account,
            32,
            32,
            string::utf8(b"Beautiful specimen!"),
            string::utf8(b"earth"),
            string::utf8(b"jade"),
            string::utf8(b""),
        );

        let account_address = signer::address_of(account);
        assert!(object::is_owner(hero, account_address), 0);
        assert!(object::is_owner(weapon, account_address), 1);
        assert!(object::is_owner(gem, account_address), 2);

        hero_equip_weapon(account, hero, weapon);
        assert!(object::is_owner(hero, account_address), 3);
        assert!(object::is_owner(weapon, object::object_address(&hero)), 4);
        assert!(object::is_owner(gem, account_address), 5);

        weapon_equip_gem(account, weapon, gem);
        assert!(object::is_owner(hero, account_address), 6);
        assert!(object::is_owner(weapon, object::object_address(&hero)), 7);
        assert!(object::is_owner(gem, object::object_address(&weapon)), 8);

        hero_unequip_weapon(account, hero, weapon);
        assert!(object::is_owner(hero, account_address), 9);
        assert!(object::is_owner(weapon, account_address), 10);
        assert!(object::is_owner(gem, object::object_address(&weapon)), 11);

        weapon_unequip_gem(account, weapon, gem);
        assert!(object::is_owner(hero, account_address), 12);
        assert!(object::is_owner(weapon, account_address), 13);
        assert!(object::is_owner(gem, account_address), 14);
    }
}