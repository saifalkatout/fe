use parser::ast;

use crate::{
    hir_def::{
        Body, BodySourceMap, Expr, ExprId, NodeStore, Partial, Pat, PatId, Stmt, StmtId,
        TrackedBodyId, TrackedItemId,
    },
    span::HirOrigin,
};

use super::FileLowerCtxt;

impl Body {
    pub(super) fn lower_ast(
        f_ctxt: &mut FileLowerCtxt,
        parent_id: TrackedItemId,
        ast: ast::Expr,
    ) -> Self {
        let bid = TrackedBodyId::ItemBody(parent_id.into());
        let mut ctxt = BodyCtxt::new(f_ctxt, bid);
        Expr::lower_ast(&mut ctxt, ast.clone());
        ctxt.build(&ast)
    }

    pub(super) fn lower_ast_nested(
        f_ctxt: &mut FileLowerCtxt,
        bid: TrackedBodyId,
        ast: ast::Expr,
    ) -> Self {
        let bid = TrackedBodyId::NestedBody(bid.into());
        let mut ctxt = BodyCtxt::new(f_ctxt, bid);
        Expr::lower_ast(&mut ctxt, ast.clone());
        ctxt.build(&ast)
    }

    pub(super) fn lower_ast_nameless(f_ctxt: &mut FileLowerCtxt<'_>, ast: ast::Expr) -> Self {
        let bid = TrackedBodyId::NamelessBody;
        let mut ctxt = BodyCtxt::new(f_ctxt, bid);
        Expr::lower_ast(&mut ctxt, ast.clone());
        ctxt.build(&ast)
    }
}

pub(super) struct BodyCtxt<'ctxt, 'db> {
    pub(super) f_ctxt: &'ctxt mut FileLowerCtxt<'db>,
    pub(super) bid: TrackedBodyId,

    pub(super) stmts: NodeStore<StmtId, Partial<Stmt>>,
    pub(super) exprs: NodeStore<ExprId, Partial<Expr>>,
    pub(super) pats: NodeStore<PatId, Partial<Pat>>,
    pub(super) source_map: BodySourceMap,
}

impl<'ctxt, 'db> BodyCtxt<'ctxt, 'db> {
    pub(super) fn push_expr(&mut self, expr: Expr, origin: HirOrigin<ast::Expr>) -> ExprId {
        let expr_id = self.exprs.push(Partial::Present(expr));
        self.source_map.expr_map.insert(expr_id, origin);

        expr_id
    }

    pub(super) fn push_invalid_expr(&mut self, origin: HirOrigin<ast::Expr>) -> ExprId {
        let expr_id = self.exprs.push(Partial::Absent);
        self.source_map.expr_map.insert(expr_id, origin);

        expr_id
    }

    pub(super) fn push_missing_expr(&mut self) -> ExprId {
        let expr_id = self.exprs.push(Partial::Absent);
        self.source_map.expr_map.insert(expr_id, HirOrigin::None);
        expr_id
    }

    pub(super) fn push_stmt(&mut self, stmt: Stmt, origin: HirOrigin<ast::Stmt>) -> StmtId {
        let stmt_id = self.stmts.push(Partial::Present(stmt));
        self.source_map.stmt_map.insert(stmt_id, origin);

        stmt_id
    }

    pub(super) fn push_pat(&mut self, pat: Pat, origin: HirOrigin<ast::Pat>) -> PatId {
        let pat_id = self.pats.push(Partial::Present(pat));
        self.source_map.pat_map.insert(pat_id, origin);
        pat_id
    }

    pub(super) fn push_missing_pat(&mut self) -> PatId {
        let pat_id = self.pats.push(Partial::Absent);
        self.source_map.pat_map.insert(pat_id, HirOrigin::None);
        pat_id
    }

    fn new(f_ctxt: &'ctxt mut FileLowerCtxt<'db>, bid: TrackedBodyId) -> Self {
        f_ctxt.enter_scope();
        Self {
            f_ctxt,
            bid,
            stmts: NodeStore::new(),
            exprs: NodeStore::new(),
            pats: NodeStore::new(),
            source_map: BodySourceMap::default(),
        }
    }

    fn build(self, ast: &ast::Expr) -> Body {
        let origin = HirOrigin::raw(ast);
        let body = Body::new(
            self.f_ctxt.db,
            self.bid,
            self.stmts,
            self.exprs,
            self.pats,
            self.f_ctxt.top_mod,
            self.source_map,
            origin,
        );

        self.f_ctxt.leave_scope(body);
        body
    }
}
