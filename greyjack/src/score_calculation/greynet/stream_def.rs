// stream_def.rs - Complete enhanced implementation with universal join system

use super::factory::ConstraintFactory;
use super::joiner::JoinerType;
use super::collectors::BaseCollector;
use super::nodes::{ImpactFn, ZeroCopyKeyFn, ZeroCopyPredicate, ZeroCopyImpactFn, ZeroCopyMapperFn};
use super::{GreynetFact, ScoreTrait, AnyTuple, constraint::ConstraintId, Result as GreynetResult};
use std::any::TypeId;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::hash::Hash;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use super::prelude::ZeroCopyFacts;

// Arity marker types
#[derive(Clone)]
pub struct Arity1;
#[derive(Clone)]
pub struct Arity2;
#[derive(Clone)]
pub struct Arity3;
#[derive(Clone)]
pub struct Arity4;
#[derive(Clone)]
pub struct Arity5;

#[derive(Clone)]
pub struct ConstraintRecipe<S: ScoreTrait> {
    pub stream_def: StreamDefinition<S>,
    pub penalty_function: super::nodes::ImpactFn<S>,
    pub constraint_id: ConstraintId,
}

static NEXT_COLLECTOR_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct CollectorSupplier {
    id: usize,
    supplier: Rc<dyn Fn() -> Box<dyn BaseCollector>>,
}

impl CollectorSupplier {
    pub fn new(supplier: impl Fn() -> Box<dyn BaseCollector> + 'static) -> Self {
        Self {
            id: NEXT_COLLECTOR_ID.fetch_add(1, Ordering::Relaxed),
            supplier: Rc::new(supplier),
        }
    }
    
    #[inline]
    pub fn create(&self) -> Box<dyn BaseCollector> {
        (self.supplier)()
    }
    
    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }
}

impl PartialEq for CollectorSupplier {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl std::hash::Hash for CollectorSupplier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl std::fmt::Debug for CollectorSupplier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollectorSupplier").field("id", &self.id).finish()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FunctionId(pub usize);

/// Type alias for tuple transformation functions
pub type ZeroCopyTupleMapperFn = Rc<dyn Fn(&dyn ZeroCopyFacts) -> AnyTuple>;

// PERFORMANCE FIX: Optimized RetrievalId
#[derive(Clone, Debug)]
pub enum RetrievalId<S: ScoreTrait> {
    From(TypeId, PhantomData<S>),
    Filter { source: Box<RetrievalId<S>>, predicate_id: FunctionId },
    Join { left: Box<RetrievalId<S>>, right: Box<RetrievalId<S>>, joiner: JoinerType, left_key_id: FunctionId, right_key_id: FunctionId },
    ConditionalJoin { source: Box<RetrievalId<S>>, other: Box<RetrievalId<S>>, should_exist: bool, left_key_id: FunctionId, right_key_id: FunctionId },
    Group { source: Box<RetrievalId<S>>, key_fn_id: FunctionId, collector_id: usize },
    FlatMap { source: Box<RetrievalId<S>>, mapper_fn_id: FunctionId },
    Map { source: Box<RetrievalId<S>>, mapper_fn_id: FunctionId },
    // PERFORMANCE FIX: Replace Vec<RetrievalId> with hash for Union
    Union { sources_hash: u64, source_count: usize },
    Distinct { source: Box<RetrievalId<S>> },
    GlobalAggregate { source: Box<RetrievalId<S>>, collector_id: usize },
    Scoring { source: Box<RetrievalId<S>>, impact_fn_id: FunctionId, constraint_id: ConstraintId },
}

impl<S: ScoreTrait> PartialEq for RetrievalId<S> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RetrievalId::From(a, _), RetrievalId::From(b, _)) => a == b,
            (RetrievalId::Filter { source: a, predicate_id: b }, 
             RetrievalId::Filter { source: c, predicate_id: d }) => a == c && b == d,
            (RetrievalId::Join { left: a, right: b, joiner: c, left_key_id: d, right_key_id: e },
             RetrievalId::Join { left: f, right: g, joiner: h, left_key_id: i, right_key_id: j }) => {
                a == f && b == g && c == h && d == i && e == j
            },
            (RetrievalId::ConditionalJoin { source: a, other: b, should_exist: c, left_key_id: d, right_key_id: e },
             RetrievalId::ConditionalJoin { source: f, other: g, should_exist: h, left_key_id: i, right_key_id: j }) => {
                a == f && b == g && c == h && d == i && e == j
            },
            (RetrievalId::Group { source: a, key_fn_id: b, collector_id: c },
             RetrievalId::Group { source: d, key_fn_id: e, collector_id: f }) => {
                a == d && b == e && c == f
            },
            (RetrievalId::FlatMap { source: a, mapper_fn_id: b },
             RetrievalId::FlatMap { source: c, mapper_fn_id: d }) => {
                a == c && b == d
            },
            (RetrievalId::Map { source: a, mapper_fn_id: b },
             RetrievalId::Map { source: c, mapper_fn_id: d }) => {
                a == c && b == d
            },
            // PERFORMANCE FIX: Fast union comparison using hash
            (RetrievalId::Union { sources_hash: a, source_count: b }, 
             RetrievalId::Union { sources_hash: c, source_count: d }) => {
                a == c && b == d
            },
            (RetrievalId::Distinct { source: a }, RetrievalId::Distinct { source: b }) => {
                a == b
            },
            (RetrievalId::GlobalAggregate { source: a, collector_id: b },
             RetrievalId::GlobalAggregate { source: c, collector_id: d }) => {
                a == c && b == d
            },
            (RetrievalId::Scoring { source: a, impact_fn_id: b, constraint_id: c },
             RetrievalId::Scoring { source: d, impact_fn_id: e, constraint_id: f }) => {
                a == d && b == e && c == f
            },
            _ => false,
        }
    }
}

impl<S: ScoreTrait> Eq for RetrievalId<S> {}

impl<S: ScoreTrait> Hash for RetrievalId<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            RetrievalId::From(type_id, _) => {
                type_id.hash(state);
            },
            RetrievalId::Filter { source, predicate_id } => {
                source.hash(state);
                predicate_id.hash(state);
            },
            RetrievalId::Join { left, right, joiner, left_key_id, right_key_id } => {
                left.hash(state);
                right.hash(state);
                joiner.hash(state);
                left_key_id.hash(state);
                right_key_id.hash(state);
            },
            RetrievalId::ConditionalJoin { source, other, should_exist, left_key_id, right_key_id } => {
                source.hash(state);
                other.hash(state);
                should_exist.hash(state);
                left_key_id.hash(state);
                right_key_id.hash(state);
            },
            RetrievalId::Group { source, key_fn_id, collector_id } => {
                source.hash(state);
                key_fn_id.hash(state);
                collector_id.hash(state);
            },
            RetrievalId::FlatMap { source, mapper_fn_id } => {
                source.hash(state);
                mapper_fn_id.hash(state);
            },
            RetrievalId::Map { source, mapper_fn_id } => {
                source.hash(state);
                mapper_fn_id.hash(state);
            },
            // PERFORMANCE FIX: Fast union hashing using precomputed hash
            RetrievalId::Union { sources_hash, source_count } => {
                sources_hash.hash(state);
                source_count.hash(state);
            },
            RetrievalId::Distinct { source } => {
                source.hash(state);
            },
            RetrievalId::GlobalAggregate { source, collector_id } => {
                source.hash(state);
                collector_id.hash(state);
            },
            RetrievalId::Scoring { source, impact_fn_id, constraint_id } => {
                source.hash(state);
                impact_fn_id.hash(state);
                constraint_id.hash(state);
            },
        }
    }
}

#[derive(Clone)]
pub enum StreamDefinition<S: ScoreTrait> {
    From(FromDefinition),
    Filter(FilterDefinition<S>),
    Join(JoinDefinition<S>),
    ConditionalJoin(ConditionalJoinDefinition<S>),
    Group(GroupDefinition<S>),
    FlatMap(FlatMapDefinition<S>),
    Map(MapDefinition<S>),
    Union(UnionDefinition<S>),
    Distinct(DistinctDefinition<S>),
    GlobalAggregate(GlobalAggregateDefinition<S>),
    Scoring(ScoringDefinition<S>),
}

impl<S: ScoreTrait> StreamDefinition<S> {
    pub fn get_retrieval_id(&self) -> RetrievalId<S> {
        match self {
            StreamDefinition::From(def) => RetrievalId::From(def.fact_type, PhantomData),
            StreamDefinition::Filter(def) => RetrievalId::Filter { source: Box::new(def.source.get_retrieval_id()), predicate_id: def.predicate_id.clone() },
            StreamDefinition::Join(def) => RetrievalId::Join { left: Box::new(def.left_source.get_retrieval_id()), right: Box::new(def.right_source.get_retrieval_id()), joiner: def.joiner_type, left_key_id: def.left_key_fn_id.clone(), right_key_id: def.right_key_fn_id.clone() },
            StreamDefinition::ConditionalJoin(def) => RetrievalId::ConditionalJoin { source: Box::new(def.source.get_retrieval_id()), other: Box::new(def.other.get_retrieval_id()), should_exist: def.should_exist, left_key_id: def.left_key_fn_id.clone(), right_key_id: def.right_key_fn_id.clone() },
            StreamDefinition::Group(def) => RetrievalId::Group { source: Box::new(def.source.get_retrieval_id()), key_fn_id: def.key_fn_id.clone(), collector_id: def.collector_supplier.id() },
            StreamDefinition::FlatMap(def) => RetrievalId::FlatMap { source: Box::new(def.source.get_retrieval_id()), mapper_fn_id: def.mapper_fn_id.clone() },
            StreamDefinition::Map(def) => RetrievalId::Map { source: Box::new(def.source.get_retrieval_id()), mapper_fn_id: def.mapper_fn_id.clone() },
            // PERFORMANCE FIX: Compute hash once instead of storing Vec
            StreamDefinition::Union(def) => {
                let mut hasher = DefaultHasher::new();
                for source in &def.sources {
                    source.get_retrieval_id().hash(&mut hasher);
                }
                RetrievalId::Union { 
                    sources_hash: hasher.finish(), 
                    source_count: def.sources.len() 
                }
            },
            StreamDefinition::Distinct(def) => RetrievalId::Distinct { source: Box::new(def.source.get_retrieval_id()) },
            StreamDefinition::GlobalAggregate(def) => RetrievalId::GlobalAggregate { source: Box::new(def.source.get_retrieval_id()), collector_id: def.collector_supplier.id() },
            StreamDefinition::Scoring(def) => RetrievalId::Scoring { source: Box::new(def.source.get_retrieval_id()), impact_fn_id: def.impact_fn_id.clone(), constraint_id: def.constraint_id },
        }
    }
    
    pub fn output_arity(&self) -> usize {
        match self {
            StreamDefinition::From(_) => 1,
            StreamDefinition::Filter(def) => def.source.output_arity(),
            StreamDefinition::Join(def) => def.left_source.output_arity() + def.right_source.output_arity(),
            StreamDefinition::ConditionalJoin(def) => def.source.output_arity(),
            StreamDefinition::Group(_) => 2,
            StreamDefinition::FlatMap(_) => 1,
            StreamDefinition::Map(def) => def.target_arity,
            StreamDefinition::Union(def) => def.sources.first().map_or(1, |s| s.output_arity()),
            StreamDefinition::Distinct(def) => def.source.output_arity(),
            StreamDefinition::GlobalAggregate(_) => 1,
            StreamDefinition::Scoring(def) => def.source.output_arity(),
        }
    }

    pub fn get_parent(&self) -> Option<&StreamDefinition<S>> {
        match self {
            StreamDefinition::Filter(def) => Some(&def.source),
            StreamDefinition::Group(def) => Some(&def.source),
            StreamDefinition::FlatMap(def) => Some(&def.source),
            StreamDefinition::Map(def) => Some(&def.source),
            StreamDefinition::Distinct(def) => Some(&def.source),
            StreamDefinition::GlobalAggregate(def) => Some(&def.source),
            StreamDefinition::Scoring(def) => Some(&def.source),
            _ => None,
        }
    }

    pub fn get_join_parents(&self) -> Option<(&StreamDefinition<S>, &StreamDefinition<S>)> {
        match self {
            StreamDefinition::Join(def) => Some((&def.left_source, &def.right_source)),
            StreamDefinition::ConditionalJoin(def) => Some((&def.source, &def.other)),
            _ => None,
        }
    }

    pub fn get_union_parents(&self) -> Option<&[StreamDefinition<S>]> {
        match self {
            StreamDefinition::Union(def) => Some(&def.sources),
            _ => None,
        }
    }
}

// Definition structs
#[derive(Debug, Clone)]
pub struct FromDefinition { 
    pub fact_type: TypeId, 
    pub fact_type_name: &'static str 
}

impl FromDefinition { 
    pub fn new<T: GreynetFact>() -> Self { 
        Self { 
            fact_type: TypeId::of::<T>(), 
            fact_type_name: std::any::type_name::<T>() 
        } 
    } 
}

#[derive(Clone)]
pub struct FilterDefinition<S: ScoreTrait> { 
    pub source: Box<StreamDefinition<S>>, 
    pub predicate_id: FunctionId, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> FilterDefinition<S> { 
    pub fn new(source: StreamDefinition<S>, predicate_id: FunctionId) -> Self { 
        Self { 
            source: Box::new(source), 
            predicate_id, 
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct JoinDefinition<S: ScoreTrait> { 
    pub left_source: Box<StreamDefinition<S>>, 
    pub right_source: Box<StreamDefinition<S>>, 
    pub joiner_type: JoinerType, 
    pub left_key_fn_id: FunctionId, 
    pub right_key_fn_id: FunctionId, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> JoinDefinition<S> { 
    pub fn new(source: StreamDefinition<S>, other: StreamDefinition<S>, joiner: JoinerType, left_key: FunctionId, right_key: FunctionId) -> Self { 
        Self { 
            left_source: Box::new(source), 
            right_source: Box::new(other), 
            joiner_type: joiner, 
            left_key_fn_id: left_key, 
            right_key_fn_id: right_key, 
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct ConditionalJoinDefinition<S: ScoreTrait> { 
    pub source: Box<StreamDefinition<S>>, 
    pub other: Box<StreamDefinition<S>>, 
    pub should_exist: bool, 
    pub left_key_fn_id: FunctionId, 
    pub right_key_fn_id: FunctionId, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> ConditionalJoinDefinition<S> { 
    pub fn new(source: StreamDefinition<S>, other: StreamDefinition<S>, should_exist: bool, left_key: FunctionId, right_key: FunctionId) -> Self { 
        Self { 
            source: Box::new(source), 
            other: Box::new(other), 
            should_exist, 
            left_key_fn_id: left_key, 
            right_key_fn_id: right_key, 
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct GroupDefinition<S: ScoreTrait> { 
    pub source: Box<StreamDefinition<S>>, 
    pub key_fn_id: FunctionId, 
    pub collector_supplier: CollectorSupplier, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> GroupDefinition<S> { 
    pub fn new(source: StreamDefinition<S>, key_fn_id: FunctionId, collector_supplier: CollectorSupplier) -> Self { 
        Self { 
            source: Box::new(source), 
            key_fn_id, 
            collector_supplier, 
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct ScoringDefinition<S: ScoreTrait> { 
    pub source: Box<StreamDefinition<S>>, 
    pub impact_fn_id: FunctionId, 
    pub constraint_id: ConstraintId, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> ScoringDefinition<S> { 
    pub fn new(source: StreamDefinition<S>, impact_fn_id: FunctionId, constraint_id: ConstraintId) -> Self { 
        Self { 
            source: Box::new(source), 
            impact_fn_id, 
            constraint_id, 
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct FlatMapDefinition<S: ScoreTrait> { 
    pub source: Box<StreamDefinition<S>>, 
    pub mapper_fn_id: FunctionId, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> FlatMapDefinition<S> { 
    pub fn new(source: StreamDefinition<S>, mapper_fn_id: FunctionId) -> Self { 
        Self { 
            source: Box::new(source), 
            mapper_fn_id, 
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct MapDefinition<S: ScoreTrait> { 
    pub source: Box<StreamDefinition<S>>, 
    pub mapper_fn_id: FunctionId, 
    pub target_arity: usize,
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> MapDefinition<S> { 
    pub fn new(source: StreamDefinition<S>, mapper_fn_id: FunctionId, target_arity: usize) -> Self { 
        Self { 
            source: Box::new(source), 
            mapper_fn_id, 
            target_arity,
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct UnionDefinition<S: ScoreTrait> { 
    pub sources: Vec<StreamDefinition<S>>, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> UnionDefinition<S> { 
    pub fn new(sources: Vec<StreamDefinition<S>>) -> Self { 
        Self { 
            sources, 
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct DistinctDefinition<S: ScoreTrait> { 
    pub source: Box<StreamDefinition<S>>, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> DistinctDefinition<S> { 
    pub fn new(source: StreamDefinition<S>) -> Self { 
        Self { 
            source: Box::new(source), 
            _phantom: PhantomData 
        } 
    } 
}

#[derive(Clone)]
pub struct GlobalAggregateDefinition<S: ScoreTrait> { 
    pub source: Box<StreamDefinition<S>>, 
    pub collector_supplier: CollectorSupplier, 
    pub _phantom: PhantomData<S> 
}

impl<S: ScoreTrait> GlobalAggregateDefinition<S> { 
    pub fn new(source: StreamDefinition<S>, collector_supplier: CollectorSupplier) -> Self { 
        Self { 
            source: Box::new(source), 
            collector_supplier, 
            _phantom: PhantomData 
        } 
    } 
}

pub fn extract_fact<T: GreynetFact>(tuple: &dyn ZeroCopyFacts, index: usize) -> Option<&T> {
    tuple.get_fact_ref(index).and_then(|fact| fact.as_any().downcast_ref::<T>())
}

// Universal key extraction helpers
pub mod key_extractors {
    use super::*;
    
    /// Extract a fact at a specific index and apply a key function
    pub fn at_index<T, K, F>(index: usize, key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        move |tuple| {
            extract_fact::<T>(tuple, index).map(|fact| key_fn(fact))
        }
    }
    
    /// Extract the first fact of type T
    pub fn first<T, K, F>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        at_index(0, key_fn)
    }
    
    /// Extract the second fact of type T
    pub fn second<T, K, F>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        at_index(1, key_fn)
    }
    
    /// Extract multiple facts and create a composite key
    pub fn composite<F, K>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static
    where
        F: Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static,
        K: Hash + 'static,
    {
        key_fn
    }
    
    /// Extract facts by type, searching all positions
    pub fn by_type<T, K, F>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        move |tuple| {
            // Search through all positions to find a fact of type T
            for i in 0..tuple.arity() {
                if let Some(fact) = extract_fact::<T>(tuple, i) {
                    return Some(key_fn(fact));
                }
            }
            None
        }
    }
    
    /// Convert an Option-returning key function to a required one with default
    pub fn with_default<K: Hash + Default + 'static>(
        key_fn: impl Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static
    ) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static {
        move |tuple| key_fn(tuple).unwrap_or_default()
    }
}

#[derive(Clone)]
pub struct Stream<A, S: ScoreTrait> {
    pub(super) definition: StreamDefinition<S>,
    pub(super) factory: Weak<RefCell<ConstraintFactory<S>>>,
    pub constraint_id_context: Option<ConstraintId>,
    _arity: PhantomData<A>,
    _score: PhantomData<S>,
}

impl<A, S: ScoreTrait + 'static> Stream<A, S> {
    pub fn new(definition: StreamDefinition<S>, factory: Weak<RefCell<ConstraintFactory<S>>>) -> Self {
        Self { definition, factory, constraint_id_context: None, _arity: PhantomData, _score: PhantomData }
    }

    pub fn new_with_context(definition: StreamDefinition<S>, factory: Weak<RefCell<ConstraintFactory<S>>>, context: Option<ConstraintId>) -> Self {
        Self { definition, factory, constraint_id_context: context, _arity: PhantomData, _score: PhantomData }
    }

    // UNIVERSAL JOIN OPERATIONS
    
    /// Universal join that works for any arity combinations
    pub fn join_on_universal<OtherArity, K, F1, F2>(
        self, 
        other: Stream<OtherArity, S>, 
        joiner_type: JoinerType,
        left_key_fn: F1, 
        right_key_fn: F2
    ) -> Stream<Arity2, S> // For now, always return combined arity as Arity2 for simplicity
    where
        K: Hash + 'static,
        F1: Fn(&dyn ZeroCopyFacts) -> K + 'static,
        F2: Fn(&dyn ZeroCopyFacts) -> K + 'static,
    {
        let factory_rc = self.factory.upgrade().expect("ConstraintFactory has been dropped");
        let mut factory = factory_rc.borrow_mut();
        
        let left_zc_key: ZeroCopyKeyFn = Rc::new(move |tuple| {
            let mut hasher = DefaultHasher::new();
            left_key_fn(tuple).hash(&mut hasher);
            hasher.finish()
        });
        
        let right_zc_key: ZeroCopyKeyFn = Rc::new(move |tuple| {
            let mut hasher = DefaultHasher::new();
            right_key_fn(tuple).hash(&mut hasher);
            hasher.finish()
        });
        
        let left_key_id = factory.register_zero_copy_key_fn(left_zc_key);
        let right_key_id = factory.register_zero_copy_key_fn(right_zc_key);
        
        let join_def = JoinDefinition::new(
            self.definition, 
            other.definition, 
            joiner_type, 
            FunctionId(left_key_id), 
            FunctionId(right_key_id)
        );
        
        Stream::new_with_context(
            StreamDefinition::Join(join_def), 
            self.factory, 
            self.constraint_id_context
        )
    }
    
    /// Convenience method for equality joins
    pub fn join_on<OtherArity, K, F1, F2>(
        self, 
        other: Stream<OtherArity, S>, 
        left_key_fn: F1, 
        right_key_fn: F2
    ) -> Stream<Arity2, S>
    where
        K: Hash + 'static,
        F1: Fn(&dyn ZeroCopyFacts) -> K + 'static,
        F2: Fn(&dyn ZeroCopyFacts) -> K + 'static,
    {
        self.join_on_universal(other, JoinerType::Equal, left_key_fn, right_key_fn)
    }

    pub fn join_on_group_key<OtherArity, K, F>(
        self,
        other: Stream<OtherArity, S>,
        right_key_fn: F,
    ) -> Stream<Arity2, S>
    where
        K: Hash + 'static,
        F: Fn(&dyn ZeroCopyFacts) -> K + 'static,
        A: Clone,
    {
        let factory_rc = self.factory.upgrade().expect("ConstraintFactory has been dropped");
        let mut factory = factory_rc.borrow_mut();

        // The key from the grouped stream is already a u64 hash.
        // We just need to extract it. It's the first fact in the BiTuple.
        let left_zc_key: ZeroCopyKeyFn = Rc::new(move |tuple| {
            extract_fact::<u64>(tuple, 0)
                .copied()
                .expect("Required fact of type u64 (the group key) not found at position 0")
        });

        // The key from the other stream needs to be extracted and then hashed.
        let right_zc_key: ZeroCopyKeyFn = Rc::new(move |tuple| {
            let mut hasher = DefaultHasher::new();
            right_key_fn(tuple).hash(&mut hasher);
            hasher.finish()
        });

        let left_key_id = factory.register_zero_copy_key_fn(left_zc_key);
        let right_key_id = factory.register_zero_copy_key_fn(right_zc_key);

        let join_def = JoinDefinition::new(
            self.definition,
            other.definition,
            JoinerType::Equal,
            FunctionId(left_key_id),
            FunctionId(right_key_id),
        );

        Stream::new_with_context(
            StreamDefinition::Join(join_def),
            self.factory,
            self.constraint_id_context,
        )
    }
    
    /// Conditional join (if_exists/if_not_exists) that works universally
    pub fn if_conditionally_universal<OtherArity, K, F1, F2>(
        self,
        other: Stream<OtherArity, S>,
        should_exist: bool,
        left_key_fn: F1,
        right_key_fn: F2,
    ) -> Self
    where
        K: Hash + 'static,
        F1: Fn(&dyn ZeroCopyFacts) -> K + 'static,
        F2: Fn(&dyn ZeroCopyFacts) -> K + 'static,
    {
        let factory_rc = self.factory.upgrade().expect("ConstraintFactory has been dropped");
        let mut factory = factory_rc.borrow_mut();
        
        let left_zc_key: ZeroCopyKeyFn = Rc::new(move |tuple| {
            let mut hasher = DefaultHasher::new();
            left_key_fn(tuple).hash(&mut hasher);
            hasher.finish()
        });
        
        let right_zc_key: ZeroCopyKeyFn = Rc::new(move |tuple| {
            let mut hasher = DefaultHasher::new();
            right_key_fn(tuple).hash(&mut hasher);
            hasher.finish()
        });
        
        let left_key_id = factory.register_zero_copy_key_fn(left_zc_key);
        let right_key_id = factory.register_zero_copy_key_fn(right_zc_key);
        
        let cond_def = ConditionalJoinDefinition::new(
            self.definition,
            other.definition,
            should_exist,
            FunctionId(left_key_id),
            FunctionId(right_key_id),
        );
        
        Self::new_with_context(
            StreamDefinition::ConditionalJoin(cond_def),
            self.factory,
            self.constraint_id_context,
        )
    }
    
    /// Universal if_exists
    pub fn if_exists<OtherArity, K, F1, F2>(
        self,
        other: Stream<OtherArity, S>,
        left_key_fn: F1,
        right_key_fn: F2,
    ) -> Self
    where
        K: Hash + 'static,
        F1: Fn(&dyn ZeroCopyFacts) -> K + 'static,
        F2: Fn(&dyn ZeroCopyFacts) -> K + 'static,
    {
        self.if_conditionally_universal(other, true, left_key_fn, right_key_fn)
    }
    
    /// Universal if_not_exists
    pub fn if_not_exists<OtherArity, K, F1, F2>(
        self,
        other: Stream<OtherArity, S>,
        left_key_fn: F1,
        right_key_fn: F2,
    ) -> Self
    where
        K: Hash + 'static,
        F1: Fn(&dyn ZeroCopyFacts) -> K + 'static,
        F2: Fn(&dyn ZeroCopyFacts) -> K + 'static,
    {
        self.if_conditionally_universal(other, false, left_key_fn, right_key_fn)
    }

    // OTHER STREAM OPERATIONS

    fn group_by_flex(self, key_fn: ZeroCopyKeyFn, collector_supplier: Box<dyn Fn() -> Box<dyn BaseCollector>>) -> Stream<Arity2, S> {
        let factory_rc = self.factory.upgrade().expect("ConstraintFactory has been dropped");
        let key_fn_id = factory_rc.borrow_mut().register_zero_copy_key_fn(key_fn);
        let group_def = GroupDefinition::new(self.definition, FunctionId(key_fn_id), CollectorSupplier::new(collector_supplier));
        Stream::new_with_context(StreamDefinition::Group(group_def), self.factory, self.constraint_id_context)
    }

    pub fn group_by<K, F>(self, key_fn: F, collector_supplier: Box<dyn Fn() -> Box<dyn BaseCollector>>) -> Stream<Arity2, S>
    where F: Fn(&dyn ZeroCopyFacts) -> K + 'static, K: Hash + 'static, {
        let zero_copy_key_fn: ZeroCopyKeyFn = Rc::new(move |tuple| {
            let mut h = DefaultHasher::new();
            key_fn(tuple).hash(&mut h);
            h.finish()
        });
        self.group_by_flex(zero_copy_key_fn, collector_supplier)
    }

    fn flat_map_flex<F>(self, mapper: F) -> Stream<Arity1, S>
    where
        F: Fn(&dyn ZeroCopyFacts) -> Vec<Rc<dyn GreynetFact>> + 'static,
    {
        let factory_rc = self.factory.upgrade().expect("ConstraintFactory has been dropped");
        let zero_copy_mapper: ZeroCopyMapperFn = Rc::new(mapper);
        let mapper_fn_id = factory_rc.borrow_mut().register_zero_copy_mapper(zero_copy_mapper);
        let flatmap_def = FlatMapDefinition::new(self.definition, FunctionId(mapper_fn_id));
        Stream::new_with_context(StreamDefinition::FlatMap(flatmap_def), self.factory, self.constraint_id_context)
    }

    pub fn flat_map<F>(self, mapper: F) -> Stream<Arity1, S>
    where
        F: Fn(&dyn ZeroCopyFacts) -> Vec<Rc<dyn GreynetFact>> + 'static,
    {
        self.flat_map_flex(mapper)
    }

    // Map operation for tuple transformation
    fn map_flex<F, TargetArity>(self, mapper: F) -> Stream<TargetArity, S>
    where
        F: Fn(&dyn ZeroCopyFacts) -> AnyTuple + 'static,
    {
        let factory_rc = self.factory.upgrade().expect("ConstraintFactory has been dropped");
        let zero_copy_tuple_mapper: ZeroCopyTupleMapperFn = Rc::new(mapper);
        let mapper_fn_id = factory_rc.borrow_mut().register_zero_copy_tuple_mapper(zero_copy_tuple_mapper);
        
        // Determine target arity based on type parameter
        let target_arity = match std::any::type_name::<TargetArity>() {
            name if name.contains("Arity1") => 1,
            name if name.contains("Arity2") => 2,
            name if name.contains("Arity3") => 3,
            name if name.contains("Arity4") => 4,
            name if name.contains("Arity5") => 5,
            _ => 1, // Default fallback
        };
        
        let map_def = MapDefinition::new(self.definition, FunctionId(mapper_fn_id), target_arity);
        Stream::new_with_context(StreamDefinition::Map(map_def), self.factory, self.constraint_id_context)
    }

    // Map operations for different target arities
    pub fn map<F>(self, mapper: F) -> Stream<Arity1, S>
    where
        F: Fn(&dyn ZeroCopyFacts) -> Rc<dyn GreynetFact> + 'static,
    {
        let adapted_mapper = move |tuple: &dyn ZeroCopyFacts| {
            AnyTuple::Uni(super::tuple::UniTuple::new(mapper(tuple)))
        };
        self.map_flex::<_, Arity1>(adapted_mapper)
    }

    pub fn map_to_pair<F>(self, mapper: F) -> Stream<Arity2, S>
    where
        F: Fn(&dyn ZeroCopyFacts) -> (Rc<dyn GreynetFact>, Rc<dyn GreynetFact>) + 'static,
    {
        let adapted_mapper = move |tuple: &dyn ZeroCopyFacts| {
            let (a, b) = mapper(tuple);
            AnyTuple::Bi(super::tuple::BiTuple::new(a, b))
        };
        self.map_flex::<_, Arity2>(adapted_mapper)
    }

    // Union operation for merging streams
    pub fn union(self, other: Stream<A, S>) -> Self {
        let sources = vec![self.definition, other.definition];
        let union_def = UnionDefinition::new(sources);
        Self::new_with_context(StreamDefinition::Union(union_def), self.factory, self.constraint_id_context)
    }

    // Distinct operation for unique tuples
    pub fn distinct(self) -> Self {
        let distinct_def = DistinctDefinition::new(self.definition);
        Self::new_with_context(StreamDefinition::Distinct(distinct_def), self.factory, self.constraint_id_context)
    }

    // Global aggregation
    pub fn aggregate(self, collector_supplier: Box<dyn Fn() -> Box<dyn BaseCollector>>) -> Stream<Arity1, S> {
        let global_agg_def = GlobalAggregateDefinition::new(self.definition, CollectorSupplier::new(collector_supplier));
        Stream::new_with_context(StreamDefinition::GlobalAggregate(global_agg_def), self.factory, self.constraint_id_context)
    }

    pub fn penalize(self, penalty_function: impl Fn(&dyn ZeroCopyFacts) -> S + 'static) -> ConstraintRecipe<S> {
        let constraint_id = self.constraint_id_context.expect("penalize() called without a constraint context. Use ConstraintBuilder::add_constraint() first.");
        self.penalize_with_id(constraint_id, penalty_function)
    }

    pub fn penalize_with_id(self, constraint_id: ConstraintId, penalty_function: impl Fn(&dyn ZeroCopyFacts) -> S + 'static) -> ConstraintRecipe<S> {
        let factory_rc = self.factory.upgrade().expect("ConstraintFactory has been dropped");
        let zero_copy_impact_fn: ZeroCopyImpactFn<S> = Rc::new(penalty_function);
        let impact_fn_id = factory_rc.borrow_mut().register_zero_copy_impact_fn(zero_copy_impact_fn.clone());
        let recipe = ConstraintRecipe { stream_def: self.definition, penalty_function: ImpactFn(zero_copy_impact_fn), constraint_id };
        factory_rc.borrow_mut().add_constraint_def(recipe.clone());
        recipe
    }
    
    pub fn filter<F>(self, predicate: F) -> Self 
    where F: Fn(&dyn ZeroCopyFacts) -> bool + 'static, 
    {
        let factory_rc = self.factory.upgrade().expect("ConstraintFactory has been dropped");
        let zero_copy_predicate: ZeroCopyPredicate = Rc::new(predicate);
        let predicate_id = factory_rc.borrow_mut().register_zero_copy_predicate(zero_copy_predicate);
        let filter_def = FilterDefinition::new(self.definition, FunctionId(predicate_id));
        Self::new_with_context(StreamDefinition::Filter(filter_def), self.factory, self.constraint_id_context)
    }
}

// Convenience macros for common join patterns
#[macro_export]
macro_rules! join_on_fact {
    ($stream:expr, $other:expr, $left_type:ty, $left_idx:expr, $left_key:expr, $right_type:ty, $right_idx:expr, $right_key:expr) => {
        $stream.join_on(
            $other,
            key_extractors::with_default(key_extractors::at_index::<$left_type, _, _>($left_idx, $left_key)),
            key_extractors::with_default(key_extractors::at_index::<$right_type, _, _>($right_idx, $right_key)),
        )
    };
}

#[macro_export]
macro_rules! join_on_first {
    ($stream:expr, $other:expr, $left_type:ty, $left_key:expr, $right_type:ty, $right_key:expr) => {
        $stream.join_on(
            $other,
            key_extractors::with_default(key_extractors::first::<$left_type, _, _>($left_key)),
            key_extractors::with_default(key_extractors::first::<$right_type, _, _>($right_key)),
        )
    };
}

#[macro_export]
macro_rules! if_exists_on_first {
    ($stream:expr, $other:expr, $left_type:ty, $left_key:expr, $right_type:ty, $right_key:expr) => {
        $stream.if_exists(
            $other,
            key_extractors::with_default(key_extractors::first::<$left_type, _, _>($left_key)),
            key_extractors::with_default(key_extractors::first::<$right_type, _, _>($right_key)),
        )
    };
}

#[macro_export]
macro_rules! if_not_exists_on_first {
    ($stream:expr, $other:expr, $left_type:ty, $left_key:expr, $right_type:ty, $right_key:expr) => {
        $stream.if_not_exists(
            $other,
            key_extractors::with_default(key_extractors::first::<$left_type, _, _>($left_key)),
            key_extractors::with_default(key_extractors::first::<$right_type, _, _>($right_key)),
        )
    };
}

// Simplified key extraction system - no more monstrous constructions!

// APPROACH 1: Direct convenience functions that return required keys
pub mod key {
    use super::*;
    
    /// Extract key from first fact of type T (panics if missing - use for required facts)
    pub fn first<T, K, F>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        move |tuple| {
            extract_fact::<T>(tuple, 0)
                .map(|fact| key_fn(fact))
                .expect("Required fact not found at position 0")
        }
    }
    
    /// Extract key from second fact of type T
    pub fn second<T, K, F>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        move |tuple| {
            extract_fact::<T>(tuple, 1)
                .map(|fact| key_fn(fact))
                .expect("Required fact not found at position 1")
        }
    }
    
    /// Extract key from fact at specific index
    pub fn at<T, K, F>(index: usize, key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        move |tuple| {
            extract_fact::<T>(tuple, index)
                .map(|fact| key_fn(fact))
                .unwrap_or_else(|| panic!("Required fact not found at position {}", index))
        }
    }
    
    /// Safe versions that return Option (for optional facts)
    pub mod optional {
        use super::*;
        
        pub fn first<T, K, F>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static
        where
            T: GreynetFact,
            K: Hash + 'static,
            F: Fn(&T) -> K + 'static,
        {
            move |tuple| extract_fact::<T>(tuple, 0).map(|fact| key_fn(fact))
        }
        
        pub fn at<T, K, F>(index: usize, key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> Option<K> + 'static
        where
            T: GreynetFact,
            K: Hash + 'static,
            F: Fn(&T) -> K + 'static,
        {
            move |tuple| extract_fact::<T>(tuple, index).map(|fact| key_fn(fact))
        }
    }
    
    /// Versions with default values (for facts that might be missing)
    pub mod with_default {
        use super::*;
        
        pub fn first<T, K, F>(key_fn: F, default: K) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
        where
            T: GreynetFact,
            K: Hash + Clone + 'static,
            F: Fn(&T) -> K + 'static,
        {
            move |tuple| {
                extract_fact::<T>(tuple, 0)
                    .map(|fact| key_fn(fact))
                    .unwrap_or_else(|| default.clone())
            }
        }
        
        pub fn at<T, K, F>(index: usize, key_fn: F, default: K) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
        where
            T: GreynetFact,
            K: Hash + Clone + 'static,
            F: Fn(&T) -> K + 'static,
        {
            move |tuple| {
                extract_fact::<T>(tuple, index)
                    .map(|fact| key_fn(fact))
                    .unwrap_or_else(|| default.clone())
            }
        }
    }
    
    /// For Default types, automatically use Default::default()
    pub mod auto_default {
        use super::*;
        
        pub fn first<T, K, F>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
        where
            T: GreynetFact,
            K: Hash + Default + 'static,
            F: Fn(&T) -> K + 'static,
        {
            move |tuple| {
                extract_fact::<T>(tuple, 0)
                    .map(|fact| key_fn(fact))
                    .unwrap_or_default()
            }
        }
        
        pub fn at<T, K, F>(index: usize, key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
        where
            T: GreynetFact,
            K: Hash + Default + 'static,
            F: Fn(&T) -> K + 'static,
        {
            move |tuple| {
                extract_fact::<T>(tuple, index)
                    .map(|fact| key_fn(fact))
                    .unwrap_or_default()
            }
        }
    }
    
    /// Direct value extraction (when the fact itself is the key)
    pub fn value<T>() -> impl Fn(&dyn ZeroCopyFacts) -> T + 'static
    where
        T: GreynetFact + Clone,
    {
        |tuple| {
            extract_fact::<T>(tuple, 0)
                .cloned()
                .expect("Required fact not found at position 0")
        }
    }
    
    /// Composite keys from multiple facts
    pub fn composite<F, K>(key_fn: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
    where
        F: Fn(&dyn ZeroCopyFacts) -> K + 'static,
        K: Hash + 'static,
    {
        key_fn
    }
}

// APPROACH 2: Ultra-simple macros
#[macro_export]
macro_rules! key {
    // key!(Type, |fact| fact.field)
    ($type:ty, $fn:expr) => {
        key::first::<$type, _, _>($fn)
    };
    
    // key!(at 1, Type, |fact| fact.field)
    (at $index:expr, $type:ty, $fn:expr) => {
        key::at::<$type, _, _>($index, $fn)
    };
    
    // key!(optional Type, |fact| fact.field)
    (optional $type:ty, $fn:expr) => {
        key::optional::first::<$type, _, _>($fn)
    };
    
    // key!(default Type, |fact| fact.field)
    (default $type:ty, $fn:expr) => {
        key::auto_default::first::<$type, _, _>($fn)
    };
    
    // key!(value Type) - for when the fact itself is the key
    (value $type:ty) => {
        key::value::<$type>()
    };
}

// APPROACH 4: Method-chaining builder pattern
pub trait KeyBuilder<T> {
    fn field<K, F>(self, f: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static;
}

pub struct FirstFact<T>(PhantomData<T>);
pub struct AtIndex<T> { index: usize, _phantom: PhantomData<T> }

pub fn first<T>() -> FirstFact<T> { FirstFact(PhantomData) }
pub fn at_index<T>(index: usize) -> AtIndex<T> { AtIndex { index, _phantom: PhantomData } }

impl<T> KeyBuilder<T> for FirstFact<T> {
    fn field<K, F>(self, f: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        key::first::<T, K, F>(f)
    }
}

impl<T> KeyBuilder<T> for AtIndex<T> {
    fn field<K, F>(self, f: F) -> impl Fn(&dyn ZeroCopyFacts) -> K + 'static
    where
        T: GreynetFact,
        K: Hash + 'static,
        F: Fn(&T) -> K + 'static,
    {
        key::at::<T, K, F>(self.index, f)
    }
}