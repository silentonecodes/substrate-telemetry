use std::collections::HashMap;
use actix::prelude::*;

use crate::chain::{self, Chain, ChainId};
use crate::node::{NodeDetails, connector::Initialize};
use crate::feed::{self, FeedMessageSerializer};
use crate::feed::connector::{FeedConnector, Connected, FeedId};
use crate::util::DenseMap;

pub struct Aggregator {
    labels: HashMap<Box<str>, ChainId>,
    chains: DenseMap<Addr<Chain>>,
    feeds: DenseMap<Addr<FeedConnector>>,
    serializer: FeedMessageSerializer,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            labels: HashMap::new(),
            chains: DenseMap::new(),
            feeds: DenseMap::new(),
            serializer: FeedMessageSerializer::new(),
        }
    }

    /// Get an address to the chain actor by name. If the address is not found,
    /// or the address is disconnected (actor dropped), create a new one.
    pub fn lazy_chain(&mut self, label: Box<str>, ctx: &mut <Self as Actor>::Context) -> &Addr<Chain> {
        let (cid, found) = self.labels
            .get(&label)
            .map(|&cid| (cid, true))
            .unwrap_or_else(|| {
                let recipient = ctx.address().recipient();
                let label = label.clone();
                let cid = self.chains.add_with(move |cid| Chain::new(cid, recipient, label).start());

                (cid, false)
            });

        if !found {
            self.labels.insert(label, cid);
        }

        self.chains.get(cid).expect("Entry just created above; qed")
    }
}

impl Actor for Aggregator {
    type Context = Context<Self>;
}

/// Message sent from the NodeConnector to the Aggregator upon getting all node details
#[derive(Message)]
pub struct AddNode {
    pub node: NodeDetails,
    pub chain: Box<str>,
    pub rec: Recipient<Initialize>,
}

/// Message sent from the Chain to the Aggregator when the Chain loses all nodes
#[derive(Message)]
pub struct DropChain(pub Box<str>);

/// Message sent from the FeedConnector to the Aggregator when subscribing to a new chain
#[derive(Message)]
pub struct Subscribe {
    pub chain: Box<str>,
    pub feed: Addr<FeedConnector>,
}

/// Message sent from the FeedConnector to the Aggregator when first connected
#[derive(Message)]
pub struct Connect(pub Addr<FeedConnector>);

/// Message sent from the FeedConnector to the Aggregator when disconnecting
#[derive(Message)]
pub struct Disconnect(pub FeedId);

impl Handler<AddNode> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let AddNode { node, chain, rec } = msg;

        self.lazy_chain(chain, ctx).do_send(chain::AddNode {
            node,
            rec,
        });
    }
}

impl Handler<DropChain> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: DropChain, _: &mut Self::Context) {
        let DropChain(label) = msg;

        if let Some(cid) = self.labels.remove(&label) {
            self.chains.remove(cid);
        }

        info!("Dropped chain [{}] from the aggregator", label);
    }
}

impl Handler<Connect> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) {
        let Connect(connector) = msg;

        let fid = self.feeds.add(connector.clone());

        info!("Feed #{} connected", fid);

        connector.do_send(Connected(fid));
    }
}

impl Handler<Disconnect> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) {
        let Disconnect(fid) = msg;

        info!("Feed #{} disconnected", fid);

        self.feeds.remove(fid);
    }
}